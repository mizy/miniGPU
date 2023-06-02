
interface EventHandler {
  (event: any): void;
}
class Event {
  private eventHandlers: {
    [eventName: string]: EventHandler[];
  } = {};

  on(eventName: string, handler: EventHandler) {
    if (!this.eventHandlers[eventName]) {
      this.eventHandlers[eventName] = [];
    }
    this.eventHandlers[eventName].push(handler);
  }

  off(eventName: string, handler: EventHandler) {
    const handlers = this.eventHandlers[eventName] || [];
    const index = handlers.indexOf(handler);
    if (index >= 0) {
      handlers.splice(index, 1);
    }
  }

  fire(eventName: string, ...params: any[]) {
    const handlers = this.eventHandlers[eventName] || [];
    for (const handler of handlers) {
      handler.apply(null, params);
    }
  }

  clear() {
    this.eventHandlers = {};
  }
}
export default Event;