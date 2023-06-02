import Material from './Components/Material';
import Mesh from './Components/Mesh';
import Entity from './Entity/Entity';
import MiniGPU from './MiniGPU';
import Scene from './Scene';
import Event from './Utils/Event';

export interface IRendererConfig  {
  canvas: HTMLCanvasElement,
  miniGPU: MiniGPU,
  contextConfig?: GPUCanvasConfiguration,
}
class Renderer extends Event{
  miniGPU: MiniGPU;
  canvas: HTMLCanvasElement;
  context: GPUCanvasContext;
  timer: number = 0;
  //just for every frame
  passEncoder: GPURenderPassEncoder;
  time: number = 0;
  delta: number = 0;
  constructor(config:IRendererConfig) {
    super();
    this.miniGPU = config.miniGPU;
    this.canvas = config.canvas;
    this.context = this.canvas.getContext("webgpu");
    const canvasFormat = navigator.gpu.getPreferredCanvasFormat();
    this.context.configure({
      ...config.contextConfig,
      device: this.miniGPU.device,
      format: canvasFormat,
      alphaMode: 'premultiplied'
    });
  }

  startRender(scene: Scene) {
    this.time = performance.now();
    this.timer = requestAnimationFrame(() => {
      this.render.bind(this, scene);
      this.startRender(scene);
    });
  }

  stopRender() {
    cancelAnimationFrame(this.timer);
    this.timer = undefined;
  }

  render(scene: Scene) {
    const now = performance.now();
    this.delta = now - this.time;
    const commandEncoder = this.miniGPU.device.createCommandEncoder();
    const passEncoder = commandEncoder.beginRenderPass({
      colorAttachments: [
        {
          clearValue: { r: 0.0, g: 0.0, b: 0.0, a: 0.0},
          loadOp: 'clear',
          storeOp: 'store',
          view: this.context.getCurrentTexture().createView()
        },
      ],
    });
    this.passEncoder = passEncoder;
    const children = scene.children;
    for (let i = 0; i < children.length; i++) {
      const entity = children[i];
      const systems = entity.systems;
      for (let j = 0; j < systems.length; j++) {
        const system = systems[j];
        system.update(this);
      }
    } 
    this.miniGPU.device.queue.submit([commandEncoder.finish()]);
  }
  
}
export default Renderer;