import Component from './Component';

export interface IMaterialConfig {
  shaderText?: string,
  topology?: GPUPrimitiveTopology,
  uniforms?: Float32Array,
  pipelineDesc?: GPURenderPipelineDescriptor,
    [key: string]: any,
}
class Material extends Component{
  pipeline: GPURenderPipeline;
  shaderModule: GPUShaderModule;
  shaderText: string;
  uniformBuffer?: GPUBuffer;
  topology?: GPUPrimitiveTopology;
  bindGroup?: GPUBindGroup;
  config: IMaterialConfig;

  setData(config: IMaterialConfig) {
    this.shaderText = config.shaderText;
    this.topology = config.topology || 'triangle-list';
    this.createShaderModule();
    this.createPipeline();
    if (config.uniforms) {
      this.createBindGroup();
      this.createUniformBuffer();
    }
  }
  createShaderModule() {
    const device = this.entity.miniGPU.device;
    const shaderModule = device.createShaderModule({
      code: this.shaderText,
    });
    this.shaderModule = shaderModule;
  }

  // default pipeline
  createPipeline() {
    const device = this.entity.miniGPU.device;
    const pipeline = device.createRenderPipeline(this.config.pipelineDesc||{
      vertex: {
        module: this.shaderModule,
        entryPoint: 'vertex_main',
        buffers: [
          {
            arrayStride: 4 * 3,
            attributes: [
              {
                // position
                shaderLocation: 0,
                offset: 0,
                format: 'float32x3',
              },
            ],
          },
        ],
      },
      fragment: {
        module: this.shaderModule,
        entryPoint: 'fragment_main',
        targets: [
          {
            format: navigator.gpu.getPreferredCanvasFormat(),
          },
        ],
      },
      primitive: {
        topology: this.topology,
      },
      layout: "auto"
    });
    this.pipeline = pipeline;
  }


  createUniformBuffer() {
    const device = this.entity.miniGPU.device;
    const uniformBuffer = device.createBuffer({
      size: this.config.uniforms.byteLength,
      usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
    });
    this.uniformBuffer = uniformBuffer;
  }

  // default bindGroup
  createBindGroup() {
    const device = this.entity.miniGPU.device;
    const bindGroup = device.createBindGroup({
      layout: this.pipeline.getBindGroupLayout(0),
      entries: [
        {
          binding: 0,
          resource: {
            buffer: this.uniformBuffer,
          },
        },
      ],
    });
    this.bindGroup = bindGroup;
  }

}
export default Material;