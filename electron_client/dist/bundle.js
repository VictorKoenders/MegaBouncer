/******/ (function(modules) { // webpackBootstrap
/******/ 	// The module cache
/******/ 	var installedModules = {};
/******/
/******/ 	// The require function
/******/ 	function __webpack_require__(moduleId) {
/******/
/******/ 		// Check if module is in cache
/******/ 		if(installedModules[moduleId]) {
/******/ 			return installedModules[moduleId].exports;
/******/ 		}
/******/ 		// Create a new module (and put it into the cache)
/******/ 		var module = installedModules[moduleId] = {
/******/ 			i: moduleId,
/******/ 			l: false,
/******/ 			exports: {}
/******/ 		};
/******/
/******/ 		// Execute the module function
/******/ 		modules[moduleId].call(module.exports, module, module.exports, __webpack_require__);
/******/
/******/ 		// Flag the module as loaded
/******/ 		module.l = true;
/******/
/******/ 		// Return the exports of the module
/******/ 		return module.exports;
/******/ 	}
/******/
/******/
/******/ 	// expose the modules object (__webpack_modules__)
/******/ 	__webpack_require__.m = modules;
/******/
/******/ 	// expose the module cache
/******/ 	__webpack_require__.c = installedModules;
/******/
/******/ 	// identity function for calling harmony imports with the correct context
/******/ 	__webpack_require__.i = function(value) { return value; };
/******/
/******/ 	// define getter function for harmony exports
/******/ 	__webpack_require__.d = function(exports, name, getter) {
/******/ 		if(!__webpack_require__.o(exports, name)) {
/******/ 			Object.defineProperty(exports, name, {
/******/ 				configurable: false,
/******/ 				enumerable: true,
/******/ 				get: getter
/******/ 			});
/******/ 		}
/******/ 	};
/******/
/******/ 	// getDefaultExport function for compatibility with non-harmony modules
/******/ 	__webpack_require__.n = function(module) {
/******/ 		var getter = module && module.__esModule ?
/******/ 			function getDefault() { return module['default']; } :
/******/ 			function getModuleExports() { return module; };
/******/ 		__webpack_require__.d(getter, 'a', getter);
/******/ 		return getter;
/******/ 	};
/******/
/******/ 	// Object.prototype.hasOwnProperty.call
/******/ 	__webpack_require__.o = function(object, property) { return Object.prototype.hasOwnProperty.call(object, property); };
/******/
/******/ 	// __webpack_public_path__
/******/ 	__webpack_require__.p = "";
/******/
/******/ 	// Load entry module and return exports
/******/ 	return __webpack_require__(__webpack_require__.s = 4);
/******/ })
/************************************************************************/
/******/ ([
/* 0 */
/***/ (function(module, exports) {

module.exports = React;

/***/ }),
/* 1 */
/***/ (function(module, exports, __webpack_require__) {

"use strict";

Object.defineProperty(exports, "__esModule", { value: true });
class Listener {
}
;
class ContainerComponent {
    constructor() {
        this._connector = remote.getGlobal('connector');
        this._listeners = [];
    }
    emit(channel, value = {}) {
        this._connector.send_emit(channel, value);
    }
    register_listener(channel, callback) {
        this._listeners.push({ channel, callback });
        this._connector.on(channel, callback.bind(this));
    }
    remove_listener(channel) {
        this._connector.removeAllListeners(channel);
        this._listeners = this._listeners.filter(l => l.channel != channel);
    }
    remove_all_listeners() {
        this._listeners.forEach(l => {
            this._connector.removeAllListeners(l.channel);
        });
        this._listeners = [];
    }
    toggle_active(newstate) {
        this.active = newstate;
    }
}
exports.ContainerComponent = ContainerComponent;


/***/ }),
/* 2 */
/***/ (function(module, exports, __webpack_require__) {

"use strict";

Object.defineProperty(exports, "__esModule", { value: true });
const React = __webpack_require__(0);
const Dashboard_1 = __webpack_require__(6);
const Chat_1 = __webpack_require__(5);
const Nodes_1 = __webpack_require__(7);
class ContainerState {
}
class Container extends React.Component {
    constructor() {
        super();
        this.state = {
            components: Array(new Dashboard_1.Dashboard(), new Nodes_1.Nodes(), new Chat_1.Chat()),
            active_index: 0
        };
        this.state.components.forEach(component => {
            component.title_changed = this.component_title_changed.bind(this, component);
            component.state_changed = this.component_state_changed.bind(this, component);
        });
        window.container = this;
    }
    component_title_changed(component) {
        this.forceUpdate();
    }
    component_state_changed(component) {
        let index = this.state.components.indexOf(component);
        if (index == this.state.active_index) {
            this.forceUpdate();
        }
    }
    component_clicked(component, index, event) {
        this.state.components[this.state.active_index].toggle_active(false);
        this.state.components[index].toggle_active(true);
        this.setState((current) => (Object.assign({}, current, { active_index: index })));
    }
    renderComponent(component, index) {
        const className = index == this.state.active_index ? "active" : "";
        return React.createElement("li", { key: index, className: className, onClick: this.component_clicked.bind(this, component, index) },
            React.createElement("a", { href: "#" }, component.render_title()));
    }
    render() {
        return React.createElement("div", { className: "container-fluid" },
            React.createElement("ul", { className: "nav nav-tabs" }, this.state.components.map(this.renderComponent.bind(this))),
            this.state.components[this.state.active_index].render());
    }
}
exports.Container = Container;


/***/ }),
/* 3 */
/***/ (function(module, exports) {

module.exports = ReactDOM;

/***/ }),
/* 4 */
/***/ (function(module, exports, __webpack_require__) {

"use strict";

Object.defineProperty(exports, "__esModule", { value: true });
const React = __webpack_require__(0);
const ReactDOM = __webpack_require__(3);
const Container_1 = __webpack_require__(2);
const element = document.createElement("div");
ReactDOM.render(React.createElement(Container_1.Container, null), element);
document.body.appendChild(element);
window.onkeydown = function (ev) {
    if (ev.key == "F5") {
        document.location.reload();
    }
};


/***/ }),
/* 5 */
/***/ (function(module, exports, __webpack_require__) {

"use strict";

Object.defineProperty(exports, "__esModule", { value: true });
const React = __webpack_require__(0);
const ContainerComponent_1 = __webpack_require__(1);
class ChatState {
}
class ChatMessage {
}
class Chat extends ContainerComponent_1.ContainerComponent {
    constructor() {
        super();
        this.state = {
            count: 0,
            scrollback: [],
            input_text: ''
        };
        this.register_listener('irc.message', this.irc_message_received.bind(this));
    }
    toggle_active(newstate) {
        if (newstate) {
            this.state.count = 0;
            this.state_changed();
        }
        super.toggle_active(newstate);
    }
    render_title() {
        return React.createElement("span", null,
            "Chat",
            this.state.count ? " " : "",
            this.state.count ?
                React.createElement("span", { className: "badge" }, this.state.count)
                : "");
    }
    irc_message_received(data) {
        if (data.message.type == "privmsg") {
            var message = {
                sender: data.message.sender.name,
                message: data.message.message
            };
            if (message.sender[0] == ':')
                message.sender = message.sender.substring(1);
            this.state.scrollback.push(message);
            if (data.message.message.indexOf('Trangar') != -1) {
                this.state.count++;
                this.title_changed();
            }
            this.state_changed();
        }
    }
    tick() {
        this.state.count += 1;
        this.title_changed();
    }
    send_text(e) {
        e.preventDefault();
        this.emit('irc.send', {
            host: 'irc.esper.net',
            port: 6667,
            type: 'privmsg',
            target: 'Trangar',
            message: this.state.input_text
        });
        this.state.input_text = '';
        this.state_changed();
    }
    update_text(e) {
        this.state.input_text = e.target.value;
        this.state_changed();
    }
    render() {
        return React.createElement("div", null,
            this.state.scrollback.map((msg, index) => React.createElement("div", { key: index },
                React.createElement("b", null,
                    msg.sender,
                    ": "),
                msg.message)),
            React.createElement("div", { className: "float-bottom" },
                React.createElement("form", { onSubmit: this.send_text.bind(this) },
                    React.createElement("input", { type: "text", value: this.state.input_text, onChange: this.update_text.bind(this) }))));
    }
}
exports.Chat = Chat;


/***/ }),
/* 6 */
/***/ (function(module, exports, __webpack_require__) {

"use strict";

Object.defineProperty(exports, "__esModule", { value: true });
const React = __webpack_require__(0);
const ContainerComponent_1 = __webpack_require__(1);
class Dashboard extends ContainerComponent_1.ContainerComponent {
    render_title() {
        return React.createElement("span", null, "Dashboard");
    }
    render() {
        return React.createElement("div", null);
    }
}
exports.Dashboard = Dashboard;


/***/ }),
/* 7 */
/***/ (function(module, exports, __webpack_require__) {

"use strict";

Object.defineProperty(exports, "__esModule", { value: true });
const React = __webpack_require__(0);
const ContainerComponent_1 = __webpack_require__(1);
class Node {
}
class NodesState {
}
class Nodes extends ContainerComponent_1.ContainerComponent {
    constructor() {
        super();
        this.state = {
            nodes: []
        };
        this.register_listener("server.clients", this.clients_received);
        this.register_listener("server.client.connected", this.client_connected);
        this.register_listener("server.client.disconnected", this.client_disconnected);
        this.register_listener("*", this.any_received);
        this._connector.send_raw("get_clients", "", {});
    }
    client_connected(message) {
        console.log('connected', JSON.stringify(message));
    }
    client_disconnected(message) {
        var index = this.state.nodes.findIndex(n => n.name == message.name);
        if (index != -1) {
            this.state.nodes.splice(index, 1);
            this.state_changed();
            this.title_changed();
        }
    }
    any_received(message) {
        console.log(JSON.stringify(message));
        if (message.sender) {
            var node = this.state.nodes.find(n => n.name == message.sender);
            if (node == null) {
                console.log('creating new node for ', message.sender);
                node = { name: message.sender, sendQueue: [] };
                this.state.nodes.push(node);
                this.title_changed();
            }
            node.sendQueue.unshift(message);
            while (node.sendQueue.length > 10) {
                node.sendQueue.pop();
            }
            this.state_changed();
        }
    }
    clients_received(clients) {
        for (let name of clients.clients) {
            let client = this.state.nodes.find(n => n.name == name);
            if (client == null) {
                this.state.nodes.push({
                    name: name,
                    sendQueue: []
                });
            }
        }
        for (let client of this.state.nodes.filter(c => clients.clients.every(cl => cl != c.name))) {
            this.state.nodes.splice(this.state.nodes.indexOf(client), 1);
        }
        this.title_changed();
        this.state_changed();
    }
    render_title() {
        return React.createElement("span", null,
            "Nodes (",
            this.state.nodes.length,
            ")");
    }
    render_node(node, index) {
        return React.createElement("li", { key: index },
            React.createElement("b", null,
                node.name,
                " (",
                node.sendQueue.length,
                ")"),
            React.createElement("br", null),
            React.createElement("ul", null, node.sendQueue.map((item, index) => React.createElement("li", { key: index }, JSON.stringify(item)))));
    }
    render() {
        return React.createElement("ul", null, this.state.nodes.map(this.render_node.bind(this)));
    }
}
exports.Nodes = Nodes;


/***/ })
/******/ ]);
//# sourceMappingURL=bundle.js.map