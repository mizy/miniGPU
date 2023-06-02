import Component from '../Components/Component';
import MiniGPU from '../MiniGPU';
import System from '../System/System';

class Entity{
  componentsMap: Record<string, Component> = {};
  systems: System[] = [];
  children: Entity[] = [];
  parent: Entity | null = null;
  miniGPU: MiniGPU | null = null;
   
  addChild(child: Entity) {
    this.children.push(child);
    child.parent = this;
  }

  removeChild(child: Entity) {
    const index = this.children.indexOf(child);
    if (index > -1) {
      this.children.splice(index, 1);
    }
  }

  addSystem(system: System) {
    this.systems.push(system);
  }

}
export default Entity;