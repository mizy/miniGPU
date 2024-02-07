struct Data {
    values: array<u32, 1024>
};

@group(0)
@binding(0)
var<storage,read_write> data: Data;

@compute 
@workgroup_size(8)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let index = id.x;
    data.values[index] = index; // make a sort array by gpu
}