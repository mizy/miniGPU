# MiniGPU
MiniGPU is a simple and lightweight graphics library for Rust. It is designed to be easy to use and easy to understand. It is based on the `wgpu` library and provides a simple and easy-to-use API for rendering 3D graphics and ready for gpu compute.
now it's still in development, and the API may change.

# Features
- [x] Simple and easy-to-use API for web frontend developers
- [x] Support for rendering 2D/3D graphics
- [x] Build-in shader include
- [x] ECS architecture
- [x] Support for gpu compute

# Example

```sh
# Run the examples with the `examples` directory
cargo run --example triangle
cargo run --example image
cargo run --example objloader
```

# WebAssembly example
https://mizy.github.io/miniGPU/examples/wasm/

# wgsl shader

## bind group index
+ group(0) for material uniform
+ group(1) for camera uniform
 
use from group(10) for other uniform binding

## build-in shader include
you can use #include, #define, #ifdef, #endif in shader code,
view `src/components/materials/shaderlibs` for more detail
```wgsl
#include <CameraUniform>
#include <VertexStruct>

#define USE_CAMERA_UNIFORM
#ifdef USE_CAMERA_UNIFORM
#include #include <CameraUniform>
#endif

```
 
# todo
- [] force-directed graph layout render
- [] a simple game demo

# weakness
 for web platform, the wasm file is huge, and also dont imporve a lot for performance