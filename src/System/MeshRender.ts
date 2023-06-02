import Material from '../Components/Material';
import Mesh from '../Components/Mesh';
import Renderer from '../Renderer';
import System from './System';

class MeshRender extends System{
  update(renderer: Renderer) {
    this.renderMesh(renderer.passEncoder)
  }

  renderMesh( passEncoder: GPURenderPassEncoder) {
    const { componentsMap } = this.entity;
    if (!componentsMap.Mesh || !componentsMap.Mateiral) {
      return;
    }
    const mesh = componentsMap.Mesh as Mesh;
    const mateiral = componentsMap.Mateiral as Material;
    passEncoder.setPipeline(mateiral.pipeline);
    passEncoder.setVertexBuffer(0, mesh.vertexBuffer);
    passEncoder.setIndexBuffer(mesh.indexBuffer, 'uint16');
    if (mateiral.bindGroup) {
      passEncoder.setBindGroup(0, mateiral.bindGroup);
    }
    passEncoder.drawIndexed(mesh.indices.length, 1, 0, 0, 0);
  }
}
export default MeshRender;