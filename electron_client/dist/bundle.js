/******/ (function(modules) { // webpackBootstrap
/******/ 	// The module cache
/******/ 	var installedModules = {};
/******/
/******/ 	// The require function
/******/ 	function __webpack_require__(moduleId) {
/******/
/******/ 		// Check if module is in cache
/******/ 		if(installedModules[moduleId])
/******/ 			return installedModules[moduleId].exports;
/******/
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
var ContainerComponent = (function () {
    function ContainerComponent() {
    }
    ContainerComponent.prototype.toggle_active = function (newstate) {
        this.active = newstate;
    };
    return ContainerComponent;
}());
exports.ContainerComponent = ContainerComponent;


/***/ }),
/* 2 */
/***/ (function(module, exports, __webpack_require__) {

"use strict";

var __extends = (this && this.__extends) || (function () {
    var extendStatics = Object.setPrototypeOf ||
        ({ __proto__: [] } instanceof Array && function (d, b) { d.__proto__ = b; }) ||
        function (d, b) { for (var p in b) if (b.hasOwnProperty(p)) d[p] = b[p]; };
    return function (d, b) {
        extendStatics(d, b);
        function __() { this.constructor = d; }
        d.prototype = b === null ? Object.create(b) : (__.prototype = b.prototype, new __());
    };
})();
var __assign = (this && this.__assign) || Object.assign || function(t) {
    for (var s, i = 1, n = arguments.length; i < n; i++) {
        s = arguments[i];
        for (var p in s) if (Object.prototype.hasOwnProperty.call(s, p))
            t[p] = s[p];
    }
    return t;
};
Object.defineProperty(exports, "__esModule", { value: true });
var React = __webpack_require__(0);
var Dashboard_1 = __webpack_require__(6);
var Chat_1 = __webpack_require__(5);
var ContainerState = (function () {
    function ContainerState() {
    }
    return ContainerState;
}());
var Container = (function (_super) {
    __extends(Container, _super);
    function Container() {
        var _this = _super.call(this) || this;
        _this.state = {
            components: Array(new Dashboard_1.Dashboard(), new Chat_1.Chat()),
            active_index: 0
        };
        _this.state.components.forEach(function (component) {
            component.title_changed = _this.component_title_changed.bind(_this, component);
            component.state_changed = _this.component_state_changed.bind(_this, component);
        });
        return _this;
    }
    Container.prototype.component_title_changed = function (component) {
        this.forceUpdate();
    };
    Container.prototype.component_state_changed = function (component) {
        var index = this.state.components.indexOf(component);
        if (index == this.state.active_index) {
            this.forceUpdate();
        }
    };
    Container.prototype.component_clicked = function (component, index, event) {
        this.state.components[this.state.active_index].toggle_active(false);
        this.state.components[index].toggle_active(true);
        this.setState(function (current) { return (__assign({}, current, { active_index: index })); });
    };
    Container.prototype.renderComponent = function (component, index) {
        var className = index == this.state.active_index ? "active" : "";
        return React.createElement("li", { key: index, className: className, onClick: this.component_clicked.bind(this, component, index) },
            React.createElement("a", { href: "#" }, component.render_title()));
    };
    Container.prototype.render = function () {
        return React.createElement("div", { className: "container-fluid" },
            React.createElement("ul", { className: "nav nav-tabs" }, this.state.components.map(this.renderComponent.bind(this))),
            this.state.components[this.state.active_index].render());
    };
    return Container;
}(React.Component));
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
var React = __webpack_require__(0);
var ReactDOM = __webpack_require__(3);
var Container_1 = __webpack_require__(2);
var element = document.createElement("div");
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

var __extends = (this && this.__extends) || (function () {
    var extendStatics = Object.setPrototypeOf ||
        ({ __proto__: [] } instanceof Array && function (d, b) { d.__proto__ = b; }) ||
        function (d, b) { for (var p in b) if (b.hasOwnProperty(p)) d[p] = b[p]; };
    return function (d, b) {
        extendStatics(d, b);
        function __() { this.constructor = d; }
        d.prototype = b === null ? Object.create(b) : (__.prototype = b.prototype, new __());
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
var React = __webpack_require__(0);
var ContainerComponent_1 = __webpack_require__(1);
var ChatState = (function () {
    function ChatState() {
    }
    return ChatState;
}());
var ChatMessage = (function () {
    function ChatMessage() {
    }
    return ChatMessage;
}());
var Chat = (function (_super) {
    __extends(Chat, _super);
    function Chat() {
        var _this = _super.call(this) || this;
        _this.state = {
            count: 0,
            scrollback: [],
            input_text: ''
        };
        _this.connector = remote.getGlobal('connector');
        _this.connector.on('irc.message', _this.irc_message_received.bind(_this));
        return _this;
    }
    Chat.prototype.toggle_active = function (newstate) {
        if (newstate) {
            this.state.count = 0;
            this.state_changed();
        }
        _super.prototype.toggle_active.call(this, newstate);
    };
    Chat.prototype.render_title = function () {
        return React.createElement("span", null,
            "Chat",
            this.state.count ? " " : "",
            this.state.count ?
                React.createElement("span", { className: "badge" }, this.state.count)
                : "");
    };
    Chat.prototype.irc_message_received = function (data) {
        if (data.message.type == "privmsg") {
            console.log(JSON.stringify(data));
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
    };
    Chat.prototype.tick = function () {
        this.state.count += 1;
        this.title_changed();
    };
    Chat.prototype.send_text = function (e) {
        e.preventDefault();
        this.connector.send_emit('irc.send', {
            host: 'irc.esper.net',
            port: 6667,
            type: 'privmsg',
            target: 'Trangar',
            message: this.state.input_text
        });
        this.state.input_text = '';
        this.state_changed();
    };
    Chat.prototype.update_text = function (e) {
        this.state.input_text = e.target.value;
        this.state_changed();
    };
    Chat.prototype.render = function () {
        return React.createElement("div", null,
            this.state.scrollback.map(function (msg, index) {
                return React.createElement("div", { key: index },
                    React.createElement("b", null,
                        msg.sender,
                        ": "),
                    msg.message);
            }),
            React.createElement("div", { className: "float-bottom" },
                React.createElement("form", { onSubmit: this.send_text.bind(this) },
                    React.createElement("input", { type: "text", value: this.state.input_text, onChange: this.update_text.bind(this) }))));
    };
    return Chat;
}(ContainerComponent_1.ContainerComponent));
exports.Chat = Chat;


/***/ }),
/* 6 */
/***/ (function(module, exports, __webpack_require__) {

"use strict";

var __extends = (this && this.__extends) || (function () {
    var extendStatics = Object.setPrototypeOf ||
        ({ __proto__: [] } instanceof Array && function (d, b) { d.__proto__ = b; }) ||
        function (d, b) { for (var p in b) if (b.hasOwnProperty(p)) d[p] = b[p]; };
    return function (d, b) {
        extendStatics(d, b);
        function __() { this.constructor = d; }
        d.prototype = b === null ? Object.create(b) : (__.prototype = b.prototype, new __());
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
var React = __webpack_require__(0);
var ContainerComponent_1 = __webpack_require__(1);
var Dashboard = (function (_super) {
    __extends(Dashboard, _super);
    function Dashboard() {
        return _super !== null && _super.apply(this, arguments) || this;
    }
    Dashboard.prototype.render_title = function () {
        return React.createElement("span", null, "Dashboard");
    };
    Dashboard.prototype.render = function () {
        return React.createElement("div", null);
    };
    return Dashboard;
}(ContainerComponent_1.ContainerComponent));
exports.Dashboard = Dashboard;


/***/ })
/******/ ]);
//# sourceMappingURL=bundle.js.map