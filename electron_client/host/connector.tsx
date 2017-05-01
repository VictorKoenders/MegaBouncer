import { EventEmitter } from "events";
import * as net from "net";

const TIMEOUT_INTERVAL: number = 5000;

declare global {
    interface Array<T> {
        includes(searchElement: T): boolean;
    }
}

type ReplyCallback = (message: any) => void;

export default class Connector extends EventEmitter {
    _reply_listeners: { [key:string]: ReplyCallback };
    _data_buffer: string;
    _reserved_events: Array<string> = ['connected', 'disconnected', 'newListener', 'removeListener'];
    _connected: boolean = false;
    _writequeue: Array<any>;
    _socket: net.Socket;
    _connect_timeout: number;

    constructor() {
        super();

        this.connect();
        super.on('connected', () => {
            this._connected = true;
        });
        super.on('disconnected', () => {
            this._connected = false;
        })
        super.on('newListener', (event: any, listener: any) => {
            if (typeof event === 'string' && super.listenerCount(event) == 0 && !this._reserved_events.includes(event)) {
                this.send({
                    action: 'register_listener',
                    channel: event
                });
            }
        });
        super.on('removeListener', (event: any, listener: any) => {
            if (typeof event === 'string' && super.listenerCount(event) == 0 && !this._reserved_events.includes(event)) {
                this.send({
                    action: 'register_listener',
                    channel: event
                });
            }
        });
    }

    send_raw(action: string, channel: string, data: object) {
        this.send({
            action: action,
            channel: channel,
            data: data
        });
    }

    send_emit(channel: string, data: object) {
        this.send({
            action: 'emit',
            channel: channel,
            data: data
        });
    }

    send_with_reply(channel: string, data: object, callback: ReplyCallback) {
        function guid() {
            function s4() {
                return Math.floor((1 + Math.random()) * 0x10000).toString(16).substring(1);
            }
            return s4() + s4() + '-' + s4() + '-' + s4() + '-' + s4() + '-' + s4() + s4() + s4();
        }
        var id = guid();
        this._reply_listeners[id] = callback;
        this.send({
            action: 'emit',
            channel: channel,
            id: id,
            data: data
        });
    }

    send(msg: any) {
        this._writequeue.push(msg);
        this.process_queue();
    }

    process_queue() {
        if (!this._connected) {
            return;
        }
        let sending = this._writequeue.map(item => JSON.stringify(item)).join("\n") + "\n";
        this._socket.write(sending);
        this._writequeue = [];
    }

    connect() {
        this._socket = net.connect({
            host: 'localhost',
            port: 12345
        });
        this._socket.setEncoding('utf8');
        this._socket.unref();
        this._socket.on('close', had_error => {
            super.emit('disconnected');
            console.log('[Connector] Close! (had error? ' + (had_error ? 'yes' : 'no') + ')');
            clearTimeout(this._connect_timeout);
            this._connect_timeout = setTimeout(this.connect.bind(this), TIMEOUT_INTERVAL);
        });
        this._socket.on('connect', () => {
            super.emit('connected');
            this._writequeue.splice(0, 0, {
                action: 'identify',
                data: {
                    name: 'Electron client'
                }
            });
            var n = 1;
            super.eventNames().forEach(listener => {
                if (typeof listener === 'string' && !this._reserved_events.includes(listener)) {
                    this._writequeue.splice(n++, 0, {
                        action: 'register_listener',
                        channel: listener
                    });
                }
            });
            this.process_queue();
        });
        this._socket.on('data', data => {
            var buffer = this._data_buffer + data;
            var split = buffer.split('\n');
            for (let i = 0; i < split.length - 1; i++) {
                var json;
                try {
                    json = JSON.parse(split[i]);
                } catch (e) {
                    console.log('Could not parse', split[i], e);
                    continue;
                }
                try {
                    this.handle_action(json);
                } catch (e) {
                    console.log('Could not handle', json);
                    continue;
                }
            }
            this._data_buffer = split[split.length - 1];
        });
        this._socket.on('end', () => {
            super.emit('disconnected');
            console.log('[Connector] End!');
            clearTimeout(this._connect_timeout);
            this._connect_timeout = setTimeout(this.connect.bind(this), TIMEOUT_INTERVAL);
        });
        this._socket.on('error', error => {
            super.emit('disconnected');
            console.log('[Connector] Error! ', error);
            clearTimeout(this._connect_timeout);
            this._connect_timeout = setTimeout(this.connect.bind(this), TIMEOUT_INTERVAL);
        });
        this._socket.on('timeout', () => {
            super.emit('disconnected');
            console.log('[Connector] Timeout!');
            clearTimeout(this._connect_timeout);
            this._connect_timeout = setTimeout(this.connect.bind(this), TIMEOUT_INTERVAL);
        });
    }

    handle_action(msg: any) {
        super.emit('*', msg);
        if (msg.reply_to) {
            var id = msg.reply_to;
            if (this._reply_listeners[id]) {
                this._reply_listeners[id](msg.data);
                delete this._reply_listeners[id];
            }
            return;
        }
        switch (msg.action) {
            case 'emit':
                super.emit(msg.channel, msg.data);
                break;
        }
    }
}

