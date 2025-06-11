use crate::{
    renderer::Renderer,
    resources::mesh::{Mesh, VertexFormat, VertexPositionTexture},
};

pub struct MakePlaneConfig {
    pub width: f32,
    pub height: f32,
    pub width_segments: usize,
    pub height_segments: usize,
}

impl Default for MakePlaneConfig {
    fn default() -> Self {
        MakePlaneConfig {
            width: 1.0,
            height: 1.0,
            width_segments: 1,
            height_segments: 1,
        }
    }
}

pub fn make_plane_data(config: MakePlaneConfig) -> (Vec<f32>, Vec<u32>, Vec<f32>, Vec<f32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();

    // 计算半宽高，使平面中心位于原点
    let width_half = config.width / 2.0;
    let height_half = config.height / 2.0;

    // 细分情况：创建网格
    let segment_width = config.width / config.width_segments as f32;
    let segment_height = config.height / config.height_segments as f32;

    // 总顶点数计算
    let vertex_count_x = config.width_segments + 1;
    let vertex_count_y = config.height_segments + 1;

    // 预分配内存以提高性能
    vertices.reserve(vertex_count_x * vertex_count_y * 3);
    normals.reserve(vertex_count_x * vertex_count_y * 3);
    uvs.reserve(vertex_count_x * vertex_count_y * 2);

    // 生成顶点、法线和UV坐标
    for iy in 0..vertex_count_y {
        let y = (iy as f32 * segment_height) - height_half;

        for ix in 0..vertex_count_x {
            let x = (ix as f32 * segment_width) - width_half;

            // 顶点位置 - XY平面
            vertices.push(x);
            vertices.push(y);
            vertices.push(0.0);

            // 法线方向 - Z轴正向
            normals.push(0.0);
            normals.push(0.0);
            normals.push(1.0);

            // UV坐标 - 标准化并翻转Y轴
            uvs.push(ix as f32 / config.width_segments as f32);
            uvs.push(1.0 - (iy as f32 / config.height_segments as f32));
        }
    }

    // 生成索引
    for iy in 0..config.height_segments {
        for ix in 0..config.width_segments {
            // 计算当前格子的四个顶点索引
            let a = iy * vertex_count_x + ix;
            let b = a + 1;
            let c = a + vertex_count_x;
            let d = c + 1;

            // 两个三角形
            indices.push(a as u32);
            indices.push(b as u32);
            indices.push(d as u32);

            indices.push(a as u32);
            indices.push(d as u32);
            indices.push(c as u32);
        }
    }

    (vertices, indices, normals, uvs)
}

pub fn make_plane_mesh(config: MakePlaneConfig, renderer: &Renderer) -> Mesh {
    let (vertices, indices, normals, uvs) = make_plane_data(config);
    let mut vertexes = Vec::new();

    for i in 0..(vertices.len() / 3) {
        vertexes.push(VertexPositionTexture {
            position: [vertices[i * 3], vertices[i * 3 + 1], vertices[i * 3 + 2]],
            tex_coords: [uvs[i * 2], uvs[i * 2 + 1]],
        });
    }

    Mesh::new(
        bytemuck::cast_slice(&vertexes),
        indices,
        VertexFormat::PositionTexture,
        renderer,
    )
}

// pub fn make_image_mesh(
//     mut width: f32,
//     mut height: f32,
//     position: Vec<f32>,
//     renderer: &Renderer,
// ) -> Mesh {
//     #[repr(C)]
//     #[derive(Clone, Copy, Pod, Zeroable)]
//     struct Vertex {
//         position: [f32; 3],
//         tex_coord: [f32; 2],
//     }
//     width = width / 2.;
//     height = height / 2.;
//     //
//     // 0----->1
//     // |
//     // |
//     // |
//     // 1
//     let vertices = vec![
//         Vertex {
//             position: [position[0] - width, position[1] - height, position[2]],
//             tex_coord: [0., 1.],
//         },
//         Vertex {
//             position: [position[0] + width, position[1] - height, position[2]],
//             tex_coord: [1., 1.],
//         },
//         Vertex {
//             position: [position[0] + width, position[1] + height, position[2]],
//             tex_coord: [1., 0.0],
//         },
//         Vertex {
//             position: [position[0] - width, position[1] + height, position[2]],
//             tex_coord: [0., 0.0],
//         },
//     ];
//     let indices = vec![0, 1, 2, 2, 3, 0];
//     let vertex_buffer = renderer
//         .device
//         .create_buffer_init(&wgpu::util::BufferInitDescriptor {
//             label: Some("Vertex Buffer"),
//             contents: bytemuck::cast_slice(&vertices),
//             usage: wgpu::BufferUsages::VERTEX,
//         });
//     let index_buffer = renderer
//         .device
//         .create_buffer_init(&wgpu::util::BufferInitDescriptor {
//             label: Some("Index Buffer"),
//             contents: bytemuck::cast_slice(&indices),
//             usage: wgpu::BufferUsages::INDEX,
//         });
//     let mesh = Mesh {
//         vertex_buffer,
//         index_buffer,
//         vertex_buffer_layout: wgpu::VertexBufferLayout {
//             array_stride: 5 * std::mem::size_of::<f32>() as wgpu::BufferAddress,
//             step_mode: wgpu::VertexStepMode::Vertex,
//             attributes: &[
//                 wgpu::VertexAttribute {
//                     offset: 0,
//                     shader_location: 0,
//                     format: wgpu::VertexFormat::Float32x3,
//                 },
//                 wgpu::VertexAttribute {
//                     offset: 3 * std::mem::size_of::<f32>() as wgpu::BufferAddress,
//                     shader_location: 1,
//                     format: wgpu::VertexFormat::Float32x2,
//                 },
//             ],
//         },
//         num_indices: indices.len() as u32,
//     };
//     mesh
// }
