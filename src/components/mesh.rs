use crate::renderer::Renderer;
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VertexFormat {
    PositionOnly,          // 仅位置 (xyz)
    PositionTexture,       // 位置 + 纹理坐标 (xyz, uv)
    PositionNormal,        // 位置 + 法线 (xyz, xyz)
    PositionNormalTexture, // 位置 + 法线 + 纹理坐标 (xyz, xyz, uv)
    Custom,                // 自定义格式
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct VertexPositionNormal {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct VertexPositionNormalTexture {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coords: [f32; 2],
}
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct VertexPositionTexture {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

pub struct Mesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
    pub vertex_format: VertexFormat,
    pub vertex_buffer_layout: wgpu::VertexBufferLayout<'static>,
    pub vertex_attributes: Vec<wgpu::VertexAttribute>,
}

impl Mesh {
    pub fn new(
        vertices_contents: &[u8],
        indices: Vec<u32>,
        format: VertexFormat,
        renderer: &Renderer,
    ) -> Mesh {
        let vertex_buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: vertices_contents,
                usage: wgpu::BufferUsages::VERTEX,
            });

        let index_buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        // 根据格式创建顶点属性
        let (vertex_attributes, array_stride) = Self::create_vertex_attributes(format);

        // 创建顶点缓冲区布局
        let vertex_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: array_stride as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[], // 临时空值，后面会设置为 vertex_attributes 的引用
        };

        let mut instance = Mesh {
            vertex_buffer,
            index_buffer,
            vertex_format: format,
            vertex_buffer_layout,
            vertex_attributes,
            num_indices: indices.len() as u32,
        };

        // 更新布局中的 attributes 引用
        instance.update_layout_attributes();

        instance
    }

    pub fn new_position_only(vertices: Vec<f32>, indices: Vec<u32>, renderer: &Renderer) -> Mesh {
        let vertex_buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let index_buffer = renderer
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        // 根据格式创建顶点属性
        let (vertex_attributes, array_stride) =
            Self::create_vertex_attributes(VertexFormat::PositionOnly);

        // 创建顶点缓冲区布局
        let vertex_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: array_stride as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[], // 临时空值，后面会设置为 vertex_attributes 的引用
        };

        let mut instance = Mesh {
            vertex_buffer,
            index_buffer,
            vertex_format: VertexFormat::PositionOnly,
            vertex_buffer_layout,
            vertex_attributes,
            num_indices: indices.len() as u32,
        };

        // 更新布局中的 attributes 引用
        instance.update_layout_attributes();

        instance
    }

    fn create_vertex_attributes(format: VertexFormat) -> (Vec<wgpu::VertexAttribute>, usize) {
        let mut attributes = Vec::new();
        let mut offset = 0;
        let total_size;

        match format {
            VertexFormat::PositionOnly => {
                // 位置: float3 (x, y, z)
                attributes.push(wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                });
                total_size = 12; // 3 * 4 bytes
            }
            VertexFormat::PositionTexture => {
                // 位置: float3 (x, y, z)
                attributes.push(wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                });
                offset += 12;

                // 纹理坐标: float2 (u, v)
                attributes.push(wgpu::VertexAttribute {
                    offset,
                    shader_location: 2, // 注意这里使用 location 2，与我们的 VertexInput 结构匹配
                    format: wgpu::VertexFormat::Float32x2,
                });
                total_size = 20; // 3 * 4 + 2 * 4 bytes
            }
            VertexFormat::PositionNormal => {
                // 位置: float3 (x, y, z)
                attributes.push(wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                });
                offset += 12;

                // 法线: float3 (nx, ny, nz)
                attributes.push(wgpu::VertexAttribute {
                    offset,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                });
                total_size = 24; // 3 * 4 + 3 * 4 bytes
            }
            VertexFormat::PositionNormalTexture => {
                // 位置: float3 (x, y, z)
                attributes.push(wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                });
                offset += 12;

                // 法线: float3 (nx, ny, nz)
                attributes.push(wgpu::VertexAttribute {
                    offset,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                });
                offset += 12;

                // 纹理坐标: float2 (u, v)
                attributes.push(wgpu::VertexAttribute {
                    offset,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                });
                total_size = 32; // 3 * 4 + 3 * 4 + 2 * 4 bytes
            }
            VertexFormat::Custom => {
                // 自定义格式，需要调用 set_custom_attributes 方法设置
                total_size = 0;
            }
        }

        (attributes, total_size)
    }

    // 添加设置自定义属性的方法
    pub fn set_custom_attributes(&mut self, attributes: Vec<wgpu::VertexAttribute>, stride: usize) {
        self.vertex_attributes = attributes;
        self.vertex_format = VertexFormat::Custom;
        self.vertex_buffer_layout.array_stride = stride as wgpu::BufferAddress;
        self.update_layout_attributes();
    }

    // 更新 vertex_buffer_layout 中的 attributes 引用
    fn update_layout_attributes(&mut self) {
        // 由于 Rust 的生命周期限制，我们需要这个技巧：
        // 1. 首先获取 attributes 的裸指针
        let ptr = self.vertex_attributes.as_ptr();
        let len = self.vertex_attributes.len();

        // 2. 将裸指针和长度转换为切片引用
        // 安全性说明：这个指针来自我们自己的 Vec，
        // 且我们确保 vertex_attributes 在 vertex_buffer_layout 的生命周期内有效
        let attributes_slice = unsafe { std::slice::from_raw_parts(ptr, len) };

        // 3. 更新 vertex_buffer_layout 中的 attributes 字段
        // 这只是一个引用，不会移动或删除数据
        self.vertex_buffer_layout.attributes = attributes_slice;
    }

    pub fn get_buffer_layout(&self) -> wgpu::VertexBufferLayout {
        self.vertex_buffer_layout.clone()
    }

    pub fn get_vertex_format(&self) -> VertexFormat {
        self.vertex_format
    }
}
