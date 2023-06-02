import Material from '../Components/Material';
import Mesh from '../Components/Mesh';
import MeshRender from '../System/MeshRender';
import Entity from './Entity';

class MeshEntity extends Entity{
  constructor() {
    super();
    this.componentsMap['Mesh'] = new Mesh();
    this.componentsMap['Material'] = new Material();
    this.addSystem(new MeshRender(this));
  }
}
export default MeshEntity;