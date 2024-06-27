use std::borrow::Cow;

use wgpu::util::DeviceExt;

fn main() {
    pollster::block_on(run_compute());
    print!("Hello, world!");
}

async fn run_compute() {
    let instance = wgpu::Instance::default();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await
        .unwrap();
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor::default(), None)
        .await
        .unwrap();

    let cs = include_str!("shader.comp.wgsl");
    let cs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Compute Shader"),
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(cs)),
    });
    let (buffer, bind_group_layout, bind_group) = make_data(&device);

    let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Compute Pipeline"),
        layout: Some(
            &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            }),
        ),
        module: &cs_module,
        entry_point: "main",
        compilation_options:  Default::default(),
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Compute Encoder"),
    });
    {
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
        cpass.set_pipeline(&compute_pipeline);
        cpass.set_bind_group(0, &bind_group, &[]);
        cpass.insert_debug_marker("compute demo");
        cpass.dispatch_workgroups(2, 1, 1);
    }
    let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: buffer.size(),
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    encoder.copy_buffer_to_buffer(&buffer, 0, &staging_buffer, 0, buffer.size());
    queue.submit(Some(encoder.finish()));

    // read the result
    let buffer_slice = staging_buffer.slice(..);
    buffer_slice.map_async(wgpu::MapMode::Read, |res| match res {
        Ok(_) => {
            println!("success");
        }
        Err(e) => {
            eprintln!("failed to map buffer: {:?}", e);
        }
    });
    device.poll(wgpu::Maintain::Wait);
    let view = buffer_slice.get_mapped_range();
    let data: &[u32] = bytemuck::cast_slice(&view);
    println!("length of data: {}", data.len());
    println!("top 10 of data: {:?}", &data[..100]);
}

fn make_data(device: &wgpu::Device) -> (wgpu::Buffer, wgpu::BindGroupLayout, wgpu::BindGroup) {
    // 创建一个数据缓冲区
    let data = [0u32; 1024];
    let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Data Buffer"),
        contents: bytemuck::cast_slice(&data),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    });

    // 创建一个绑定布局
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Bind Group Layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(4096),
            },
            count: None,
        }],
    });

    // 创建一个绑定组
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Bind Group"),
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }],
    });
    (buffer, bind_group_layout, bind_group)
}
