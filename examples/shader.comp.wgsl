struct Data {
    values: array<u32, 1024>
};

@group(0)
@binding(0)
var<storage,read_write> data: Data;

@compute 
@workgroup_size(1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    data.values[id.x] = id.x; // make a sort array by gpu
}