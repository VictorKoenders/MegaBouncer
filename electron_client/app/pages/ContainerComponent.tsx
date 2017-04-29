interface EventEmitter {
    addListener(event: string | symbol, listener: Function): this;
    on(event: string | symbol, listener: Function): this;
    once(event: string | symbol, listener: Function): this;
    prependListener(event: string | symbol, listener: Function): this;
    prependOnceListener(event: string | symbol, listener: Function): this;
    removeListener(event: string | symbol, listener: Function): this;
    removeAllListeners(event?: string | symbol): this;
    setMaxListeners(n: number): this;
    getMaxListeners(): number;
    listeners(event: string | symbol): Function[];
    //emit(event: string | symbol, ...args: any[]): boolean;
    eventNames(): (string | symbol)[];
    listenerCount(type: string | symbol): number;
    
    send_emit(channel: string, data: object): void;
    send_raw(action: string, channel: string, data: object): void;
}

declare var remote: {
    getGlobal: (name: string) => EventEmitter,
};

class Listener {
    channel: string;
    callback: Function;
};

export abstract class ContainerComponent {
    _connector: EventEmitter;
    _listeners: Array<Listener>;
    constructor() {
        this._connector = remote.getGlobal('connector');
        this._listeners = [];
    }

    emit(channel: string, value: object = {}) {
        this._connector.send_emit(channel, value);
    }

    register_listener(channel: string, callback: Function) {
        this._listeners.push({ channel, callback });
        this._connector.on(channel, callback.bind(this));
    }

    remove_listener(channel: string) {
        this._connector.removeAllListeners(channel);
        this._listeners = this._listeners.filter(l => l.channel != channel);
    }

    remove_all_listeners() {
        this._listeners.forEach(l => {
            this._connector.removeAllListeners(l.channel);
        });
        this._listeners = [];
    }
    

    title_changed: () => void;
    state_changed: () => void;
    active: boolean;

    abstract render_title(): JSX.Element;
    abstract render(): JSX.Element;
    toggle_active(newstate: boolean) {
        this.active = newstate;
    }
}
