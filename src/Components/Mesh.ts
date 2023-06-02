import Component from './Component';

class Mesh extends Component{
  vetexs: Float32Array ;
  indices: Float32Array ;
  vertexBuffer: GPUBuffer;
  indexBuffer: GPUBuffer;
  setData(data: {
    vetexs: Float32Array,
    indices: Float32Array,
  }) {
    if (data.vetexs.length !== data.indices.length) {
      this.clear();
    }
    this.vetexs = data.vetexs;
    this.indices = data.indices;
    if (!this.vertexBuffer) {
      this.createBuffer();
    }
    this.writeBuffer();
  }

  createBuffer() {
    const device = this.entity.miniGPU.device;
    const vertexBuffer = device.createBuffer({
      size: this.vetexs.byteLength, // make it big enough to store vertices in
      usage: GPUBufferUsage.VERTEX | GPUBufferUsage.COPY_DST,
    });
    this.vertexBuffer = vertexBuffer;
    const indexBuffer = device.createBuffer({
      size: this.indices.byteLength, // make it big enough to store vertices in
      usage: GPUBufferUsage.INDEX | GPUBufferUsage.COPY_DST,
    });
    this.indexBuffer = indexBuffer;
    this.writeBuffer();
  }

  writeBuffer() {
    const device = this.entity.miniGPU.device;
    device.queue.writeBuffer(this.vertexBuffer, 0, this.vetexs, 0, this.vetexs.length);
    device.queue.writeBuffer(this.indexBuffer, 0, this.indices, 0, this.indices.length);
  }

  clear() {
    this.vertexBuffer?.destroy();
    this.indexBuffer?.destroy();
    this.vertexBuffer = undefined;
    this.indexBuffer = undefined;
  }
}
export default Mesh;