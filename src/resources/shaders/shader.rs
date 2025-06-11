use regex::Regex;
use std::collections::HashMap;

use super::shaderlib;

pub struct ShaderParser {
    pub defines: HashMap<String, String>,
    ifdef_stack: Vec<IfdefState>,
    define_regex: Regex,
    define_val_regex: Regex,
    ifdef_regex: Regex,
    endif_regex: Regex,
    include_regex: Regex,
    else_regex: Regex,
    elseif_regex: Regex,
}
// 用于表示条件编译块的状态
struct IfdefState {
    name: String,
    is_active: bool,   // 当前块是否处于激活状态
    has_matched: bool, // 当前 if/elif 链中是否已经有满足的条件
}

impl ShaderParser {
    pub fn new() -> ShaderParser {
        ShaderParser {
            defines: HashMap::new(),
            ifdef_stack: Vec::new(),
            define_regex: Regex::new(r"^#define\s+(\w+)").unwrap(),
            define_val_regex: Regex::new(r"^#define\s+(\w+)\s+(\w+)").unwrap(),
            ifdef_regex: Regex::new(r"#ifdef\s+(\w+)").unwrap(),
            elseif_regex: Regex::new(r"#elseif\s+(\w+)").unwrap(),
            else_regex: Regex::new(r"#else").unwrap(),
            endif_regex: Regex::new(r"#endif").unwrap(),
            include_regex: Regex::new(r"#include\s+<(\w+)>").unwrap(),
        }
    }

    // add #define #ifdef #endif #include <>  syntax
    pub fn parse_shader(&mut self, shader: &str) -> String {
        // parse shader for each line
        let lines = shader.split('\n');
        let line_index = 0;
        let mut res: Vec<String> = Vec::new();
        // put ratio of size
        for line in lines {
            let res_line = self.handle_line(line, line_index);
            if res_line.is_some() {
                res.push(res_line.unwrap());
            }
        }
        return res.join("\n");
    }

    pub fn handle_line(&mut self, line: &str, line_index: usize) -> Option<String> {
        // check define val
        let define_val_vals = self.define_val_regex.captures(line.trim());
        if define_val_vals.is_some() {
            let values = define_val_vals.unwrap();
            let define_val = values.get(1).unwrap().as_str();
            let define_val_val = values.get(2).unwrap().as_str();
            self.defines
                .insert(define_val.to_string(), define_val_val.to_string());
            return None;
        }

        // check for #define
        let define_vals = self.define_regex.captures(line.trim());
        if define_vals.is_some() {
            let define_val = define_vals.unwrap().get(1).unwrap().as_str();
            self.defines
                .insert(define_val.to_string(), "true".to_string());
            return None;
        }

        // check for #ifdef
        let ifdef_vals = self.ifdef_regex.captures(line.trim());
        if ifdef_vals.is_some() {
            let ifdef_val = ifdef_vals.unwrap().get(1).unwrap().as_str();
            let is_defined = self.defines.contains_key(ifdef_val);

            self.ifdef_stack.push(IfdefState {
                name: ifdef_val.to_string(),
                is_active: is_defined,
                has_matched: is_defined,
            });
            return None;
        }

        // check for #elseif
        let elseif_vals = self.elseif_regex.captures(line.trim());
        if elseif_vals.is_some() {
            if let Some(current_state) = self.ifdef_stack.last_mut() {
                // 只有当前面的条件都不满足时，才检查当前条件
                if !current_state.has_matched {
                    let elseif_val = elseif_vals.unwrap().get(1).unwrap().as_str();
                    let is_defined = self.defines.contains_key(elseif_val);

                    current_state.is_active = is_defined;
                    if is_defined {
                        current_state.has_matched = true;
                    }
                } else {
                    // 已经有一个条件满足了，所以这个分支不活跃
                    current_state.is_active = false;
                }
            }
            return None;
        }

        // check for #else
        let else_vals = self.else_regex.captures(line.trim());
        if else_vals.is_some() {
            if let Some(current_state) = self.ifdef_stack.last_mut() {
                // else 只在没有匹配到任何条件时激活
                current_state.is_active = !current_state.has_matched;
                if current_state.is_active {
                    current_state.has_matched = true;
                }
            }
            return None;
        }

        // check for #endif
        let endif_vals = self.endif_regex.captures(line.trim());
        if endif_vals.is_some() {
            if self.ifdef_stack.len() != 0 {
                self.ifdef_stack.pop();
            }
            return None;
        }

        // check for #include
        let include_vals = self.include_regex.captures(line.trim());
        if include_vals.is_some() {
            let include_val = include_vals.unwrap().get(1).unwrap().as_str();
            if shaderlib::SHADER_LIB.contains_key(include_val) {
                let shader = shaderlib::SHADER_LIB.get(include_val).unwrap();
                let mut parser = ShaderParser::new();
                parser.defines = self.defines.clone();
                let res = parser.parse_shader(shader);
                return Some(res);
            } else {
                panic!(
                    "Shader include not found: {}, line_index:{}",
                    include_val, line_index
                );
            }
        }

        // match normal line
        for state in &self.ifdef_stack {
            if !state.is_active {
                return None; // 如果在任何未激活的块中，则跳过这一行
            }
        }

        Some(line.to_string())
        // check for #include
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_define_parse() {
        let shader = r#"
        #define TEST
        #define TEST2
        #define TEST3
        "#;
        let mut parser = ShaderParser::new();
        let res = parser.parse_shader(shader);
        assert_eq!(res, "\n\n\n");
    }

    #[test]
    fn test_ifdef_parse() {
        let shader = r#"
        #define TEST
        #ifdef TEST
        1
        #define TEST2
        #endif
        #ifdef TEST2
        2
        #define TEST3
        #endif
        "#;
        let mut parser = ShaderParser::new();
        let res = parser.parse_shader(shader);
        assert_eq!(res, "\n\n\n\n\n\n");
    }
}
