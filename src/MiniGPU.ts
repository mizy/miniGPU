import Renderer, { IRendererConfig } from './Renderer';
export interface IMiniGPUConfig extends IRendererConfig{
  
}
class MiniGPU {
  status = 'init'
  adapter: GPUAdapter 
  device: GPUDevice 
  renderer:Renderer
  constructor(config:IMiniGPUConfig) {
    this.init();
    if (config.canvas) {
      this.renderer = new Renderer({
        miniGPU: this,
        ...config,
      });
    }
  }

  async init() {
    // 1: request adapter and device
    if (!navigator.gpu) {
      throw Error('WebGPU not supported.');
    }
    const adapter = await navigator.gpu.requestAdapter();
    if (!adapter) {
      throw Error('Couldn\'t request WebGPU adapter.');
    }
    this.adapter = adapter;
    this.device = await adapter.requestDevice();
    this.status = 'ready'
  }
}
export default MiniGPU;