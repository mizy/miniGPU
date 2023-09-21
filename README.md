# bind group index
+ group(0) for material uniform
+ group(1) for camera uniform
 
use from group(10) for other uniform binding
# build-in shader binding

- camera

```wgsl
struct CameraUniform {
    view_projection: mat4x4<f32>,
    projection_matrix: mat4x4<f32>,
    view_matrix: mat4x4<f32>,
}
@group(1) @binding(0) // 1.
var<uniform> camera: CameraUniform;

```

- location order

```
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coord: vec2<f32>,
}

struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
};
```
