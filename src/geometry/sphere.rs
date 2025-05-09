use glam::Vec3;

use crate::{
    components::mesh::{Mesh, VertexFormat, VertexPositionNormalTexture},
    renderer::Renderer,
};

pub struct MakeSphereConfig {
    pub radius: f32,
    pub width_segments: usize,
    pub height_segments: usize,
    pub phi_start: f32,
    pub phi_length: f32,
    pub theta_start: f32,
    pub theta_length: f32,
}

impl Default for MakeSphereConfig {
    fn default() -> Self {
        MakeSphereConfig {
            radius: 1.0,
            width_segments: 12,
            height_segments: 12,
            phi_start: 0.0,
            phi_length: std::f32::consts::PI * 2.0,
            theta_start: 0.0,
            theta_length: std::f32::consts::PI,
        }
    }
}

pub fn make_sphere_data(config: MakeSphereConfig) -> (Vec<f32>, Vec<u32>, Vec<f32>, Vec<f32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut grid = Vec::new();
    let mut index = 0;

    let mut vertex = Vec3::new(0.0, 0.0, 0.0);
    let mut normal = Vec3::new(0.0, 0.0, 0.0);
    // generate vertices, normals and uvs
    for iy in 0..=config.height_segments {
        let mut vertices_row = Vec::new();
        let v = iy as f32 / config.height_segments as f32;

        // special case for the poles
        let mut u_offset = 0.0;
        if iy == 0 && config.theta_start == 0.0 {
            u_offset = 0.5 / config.width_segments as f32;
        } else if iy == config.height_segments && config.theta_length == std::f32::consts::PI {
            u_offset = -0.5 / config.width_segments as f32;
        }

        for ix in 0..=config.width_segments {
            let u = ix as f32 / config.width_segments as f32;

            vertex.x = -config.radius
                * (config.phi_start + u * config.phi_length).cos()
                * (config.theta_start + v * config.theta_length).sin();
            vertex.y = config.radius * (config.theta_start + v * config.theta_length).cos();
            vertex.z = config.radius
                * (config.phi_start + u * config.phi_length).sin()
                * (config.theta_start + v * config.theta_length).sin();

            vertices.push(vertex.x);
            vertices.push(vertex.y);
            vertices.push(vertex.z);

            // normal
            normal = vertex.normalize();
            normals.push(normal.x);
            normals.push(normal.y);
            normals.push(normal.z);

            // uv
            uvs.push(u + u_offset);
            uvs.push(1.0 - v);

            vertices_row.push(index);
            index += 1;
        }

        grid.push(vertices_row);
    }

    // indices
    for iy in 0..grid.len() - 1 {
        let row = &grid[iy];
        for ix in 0..row.len() - 1 {
            let a = grid[iy][ix + 1];
            let b = grid[iy][ix];
            let c = grid[iy + 1][ix];
            let d = grid[iy + 1][ix + 1];

            if iy != 0 || config.theta_start > 0.0 {
                indices.push(a as u32);
                indices.push(b as u32);
                indices.push(d as u32);
            }
            if iy != config.height_segments - 1 || config.theta_length < std::f32::consts::PI {
                indices.push(b as u32);
                indices.push(c as u32);
                indices.push(d as u32);
            }
        }
    }

    (vertices, indices, normals, uvs)
}

pub fn make_sphere_mesh(config: MakeSphereConfig, renderer: &Renderer) -> Mesh {
    let (vertices, indices, normals, uvs) = make_sphere_data(config);
    let mut vertexes = Vec::new();
    for i in 0..(vertices.len() / 3) {
        vertexes.push(VertexPositionNormalTexture {
            position: [vertices[i * 3], vertices[i * 3 + 1], vertices[i * 3 + 2]],
            normal: [normals[i * 3], normals[i * 3 + 1], normals[i * 3 + 2]],
            tex_coords: [uvs[i * 2], uvs[i * 2 + 1]],
        });
    }

    Mesh::new(
        bytemuck::cast_slice(&vertexes),
        indices,
        VertexFormat::PositionNormalTexture,
        renderer,
    )
}
