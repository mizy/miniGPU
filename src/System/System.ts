import Entity from '../Entity/Entity';
import Renderer from '../Renderer';

class System{
  entity: Entity;
  constructor(entity:Entity){
    this.entity = entity;
  }
  update(renderer: Renderer) { }
}
export default System;