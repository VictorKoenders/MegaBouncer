const EventEmitter = require("events").EventEmitter;
const net = require("net");

const TIMEOUT_INTERVAL = 5000;

class Connector extends EventEmitter {
    // TODO: Connect to the network and listen to specific events
    // Whenever a component registers an on(...) listener, send that listener to the server
    //  - don't register "connected" and "disconnected" with the server
    // whenever a component forgets it, remove the listener
    // Whenever we disconnect, reconnect in 5 seconds or so and send an event
    // Also send an even when we're connecting
    constructor() {
        super();
        this._reply_listeners = [];
        this._data_buffer = '';
        this._reserved_events = ['connected', 'disconnected', 'newListener', 'removeListener'];
        this._connected = false;
        this._writequeue = [];

        this.connect();
        this.on('connected', () => {
            this._connected = true;
        });
        this.on('disconnected', () => {
            this._connected = false;
        })
        this.on('newListener', (event, listener) => {
            if (typeof event === 'string' && this.listenerCount(event) == 0 && !this._reserved_events.includes(event)) {
                this.send({
                    action: 'register_listener',
                    channel: event
                });
            }
        });
        this.on('removeListener', (event, listener) => {
            if (typeof event === 'string' && this.listenerCount(event) == 0 && !this._reserved_events.includes(event)) {
                this.send({
                    action: 'register_listener',
                    channel: event
                });
            }
        })
    }

    send_raw(action, channel, data) {
        this.send({
            action: action,
            channel: channel,
            data: data
        });
    }

    send_emit(channel, data) {
        this.send({
            action: 'emit',
            channel: channel,
            data: data
        });
    }

    send_with_reply(channel, data, callback) {
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

    send(msg) {
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
            this.emit('disconnected');
            console.log('[Connector] Close! (had error? ' + (had_error ? 'yes' : 'no') + ')');
            clearTimeout(this._connect_timeout);
            this._connect_timeout = setTimeout(this.connect.bind(this), TIMEOUT_INTERVAL);
        });
        this._socket.on('connect', () => {
            this.emit('connected');
            this._writequeue.splice(0, 0, {
                action: 'identify',
                data: {
                    name: 'Electron client'
                }
            });
            var n = 1;
            this.eventNames().forEach(listener => {
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
            this.emit('disconnected');
            console.log('[Connector] End!');
            clearTimeout(this._connect_timeout);
            this._connect_timeout = setTimeout(this.connect.bind(this), TIMEOUT_INTERVAL);
        });
        this._socket.on('error', error => {
            this.emit('disconnected');
            console.log('[Connector] Error! ', error);
            clearTimeout(this._connect_timeout);
            this._connect_timeout = setTimeout(this.connect.bind(this), TIMEOUT_INTERVAL);
        });
        this._socket.on('timeout', () => {
            this.emit('disconnected');
            console.log('[Connector] Timeout!');
            clearTimeout(this._connect_timeout);
            this._connect_timeout = setTimeout(this.connect.bind(this), TIMEOUT_INTERVAL);
        });
    }

    handle_action(msg) {
        this.emit('*', msg);
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
                this.emit(msg.channel, msg.data);
                break;
        }
    }
}

module.exports = Connector;
