import Entity from '../Entity/Entity';

class Component{
  entity?: Entity
  constructor(entity?:Entity){
    this.entity = entity;
  }
}
export default Component;