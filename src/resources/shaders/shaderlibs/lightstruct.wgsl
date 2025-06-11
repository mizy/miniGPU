struct DirectionLight{
    direction: vec3<f32>,
    color: vec3<f32>,
    intensity: f32,
}

@group(1) @binding(1) var<uniform> direction_light: DirectionLight;