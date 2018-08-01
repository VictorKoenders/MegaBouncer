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
/******/ 	// define getter function for harmony exports
/******/ 	__webpack_require__.d = function(exports, name, getter) {
/******/ 		if(!__webpack_require__.o(exports, name)) {
/******/ 			Object.defineProperty(exports, name, { enumerable: true, get: getter });
/******/ 		}
/******/ 	};
/******/
/******/ 	// define __esModule on exports
/******/ 	__webpack_require__.r = function(exports) {
/******/ 		if(typeof Symbol !== 'undefined' && Symbol.toStringTag) {
/******/ 			Object.defineProperty(exports, Symbol.toStringTag, { value: 'Module' });
/******/ 		}
/******/ 		Object.defineProperty(exports, '__esModule', { value: true });
/******/ 	};
/******/
/******/ 	// create a fake namespace object
/******/ 	// mode & 1: value is a module id, require it
/******/ 	// mode & 2: merge all properties of value into the ns
/******/ 	// mode & 4: return value when already ns object
/******/ 	// mode & 8|1: behave like require
/******/ 	__webpack_require__.t = function(value, mode) {
/******/ 		if(mode & 1) value = __webpack_require__(value);
/******/ 		if(mode & 8) return value;
/******/ 		if((mode & 4) && typeof value === 'object' && value && value.__esModule) return value;
/******/ 		var ns = Object.create(null);
/******/ 		__webpack_require__.r(ns);
/******/ 		Object.defineProperty(ns, 'default', { enumerable: true, value: value });
/******/ 		if(mode & 2 && typeof value != 'string') for(var key in value) __webpack_require__.d(ns, key, function(key) { return value[key]; }.bind(null, key));
/******/ 		return ns;
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
/******/
/******/ 	// Load entry module and return exports
/******/ 	return __webpack_require__(__webpack_require__.s = "./src/index.tsx");
/******/ })
/************************************************************************/
/******/ ({

/***/ "./src/components/Root.tsx":
/*!*********************************!*\
  !*** ./src/components/Root.tsx ***!
  \*********************************/
/*! no static exports found */
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
var React = __webpack_require__(/*! react */ "react");
var node_channel_registered_1 = __webpack_require__(/*! ../handler/node.channel.registered */ "./src/handler/node.channel.registered.ts");
var node_disconnected_1 = __webpack_require__(/*! ../handler/node.disconnected */ "./src/handler/node.disconnected.ts");
var node_identified_1 = __webpack_require__(/*! ../handler/node.identified */ "./src/handler/node.identified.ts");
var node_listed_1 = __webpack_require__(/*! ../handler/node.listed */ "./src/handler/node.listed.ts");
var ui_gotten_1 = __webpack_require__(/*! ../handler/ui.gotten */ "./src/handler/ui.gotten.ts");
var handlers = {
    "node.channel.registered": node_channel_registered_1.default,
    "node.disconnected": node_disconnected_1.default,
    "node.identified": node_identified_1.default,
    "node.listed": node_listed_1.default,
    "ui.gotten": ui_gotten_1.default,
};
var Root = /** @class */ (function (_super) {
    __extends(Root, _super);
    function Root(props, context) {
        var _this = _super.call(this, props, context) || this;
        window.message_received = _this.message_received.bind(_this);
        _this.state = {
            logs: [],
            modules: [],
            active: null
        };
        return _this;
    }
    Root.prototype.message_received = function (channel, obj) {
        this.setState(function (oldstate) {
            if (handlers[channel]) {
                handlers[channel](oldstate, obj);
            }
            else {
                console.log("Channel without listener:");
                console.log(obj);
            }
            return oldstate;
        });
    };
    Root.prototype.module_clicked = function (index, e) {
        e.preventDefault();
        e.stopPropagation();
        if (index == null) {
            this.setState(function (oldState) { return ({
                active: null
            }); });
        }
        else {
            this.setState(function (oldState) { return ({
                active: oldState.modules[index]
            }); });
        }
        return false;
    };
    Root.prototype.render_link = function (module, index) {
        if (module === void 0) { module = null; }
        if (index === void 0) { index = -1; }
        var name = module === null ? "Home" : module.name;
        if (module !== null && !module.ui_loaded)
            return null;
        return (React.createElement("li", { key: index, className: this.state.active == module ? "active" : "" },
            React.createElement("a", { href: "#", onClick: this.module_clicked.bind(this, index) }, name)));
    };
    Root.prototype.render_content = function () {
        if (this.state.active == null)
            return null;
        if (!window.modules.hasOwnProperty(this.state.active.name)) {
            return (React.createElement(React.Fragment, null,
                React.createElement("b", null,
                    "Module ",
                    this.state.active.name,
                    " is not found in the list:"),
                React.createElement("br", null),
                React.createElement("ul", null, Object.keys(window.modules).map(function (m, i) { return React.createElement("li", { key: i }, m); }))));
        }
        var Connector = window.modules[this.state.active.name];
        return React.createElement(Connector, null);
    };
    Root.prototype.render = function () {
        return (React.createElement(React.Fragment, null,
            React.createElement("ul", { className: "top_bar" },
                this.render_link(),
                this.state.modules.map(this.render_link.bind(this))),
            this.render_content()));
    };
    return Root;
}(React.Component));
exports.Root = Root;


/***/ }),

/***/ "./src/handler/node.channel.registered.ts":
/*!************************************************!*\
  !*** ./src/handler/node.channel.registered.ts ***!
  \************************************************/
/*! no static exports found */
/***/ (function(module, exports, __webpack_require__) {

"use strict";

Object.defineProperty(exports, "__esModule", { value: true });
var node_listed_1 = __webpack_require__(/*! ./node.listed */ "./src/handler/node.listed.ts");
function default_1(state, obj) {
    var module = state.modules.find(function (m) { return m.id == obj.sender_id; });
    if (!module)
        return;
    if (obj.channel == "ui.get") {
        external.invoke(node_listed_1.get_emit(obj.sender_id));
    }
}
exports.default = default_1;


/***/ }),

/***/ "./src/handler/node.disconnected.ts":
/*!******************************************!*\
  !*** ./src/handler/node.disconnected.ts ***!
  \******************************************/
/*! no static exports found */
/***/ (function(module, exports, __webpack_require__) {

"use strict";

Object.defineProperty(exports, "__esModule", { value: true });
function default_1(state, obj) {
    var index = state.modules.findIndex(function (m) { return m.id == obj.id; });
    if (index >= 0) {
        if (window.modules.hasOwnProperty(state.modules[index].name)) {
            delete window.modules[state.modules[index].name];
        }
        state.modules.splice(index, 1);
    }
}
exports.default = default_1;


/***/ }),

/***/ "./src/handler/node.identified.ts":
/*!****************************************!*\
  !*** ./src/handler/node.identified.ts ***!
  \****************************************/
/*! no static exports found */
/***/ (function(module, exports, __webpack_require__) {

"use strict";

Object.defineProperty(exports, "__esModule", { value: true });
function default_1(state, obj) {
    var module = state.modules.find(function (m) { return m.id == obj.id; });
    if (module)
        return;
    module = {
        id: obj.id,
        name: obj.name,
        ui_loaded: false,
    };
    state.modules.push(module);
}
exports.default = default_1;


/***/ }),

/***/ "./src/handler/node.listed.ts":
/*!************************************!*\
  !*** ./src/handler/node.listed.ts ***!
  \************************************/
/*! no static exports found */
/***/ (function(module, exports, __webpack_require__) {

"use strict";

Object.defineProperty(exports, "__esModule", { value: true });
function get_emit(id) {
    return ("emit:" +
        JSON.stringify({
            action: "ui.get",
            target: id
        }));
}
exports.get_emit = get_emit;
function default_1(new_state, obj) {
    for (var _i = 0, _a = obj.nodes; _i < _a.length; _i++) {
        var node = _a[_i];
        if (node.channels.indexOf("ui.get") != -1) {
            new_state.modules.push({
                id: node.id,
                name: node.name,
                ui_loaded: false
            });
            var emit = get_emit(node.id);
            external.invoke(emit);
        }
    }
}
exports.default = default_1;


/***/ }),

/***/ "./src/handler/ui.gotten.ts":
/*!**********************************!*\
  !*** ./src/handler/ui.gotten.ts ***!
  \**********************************/
/*! no static exports found */
/***/ (function(module, exports, __webpack_require__) {

"use strict";

Object.defineProperty(exports, "__esModule", { value: true });
function default_1(new_state, obj) {
    var module = new_state.modules.find(function (m) { return m.id == obj.sender_id; });
    try {
        eval(obj.ui);
        module.ui_loaded = true;
    }
    catch (e) {
        console.log(e);
    }
}
exports.default = default_1;


/***/ }),

/***/ "./src/index.tsx":
/*!***********************!*\
  !*** ./src/index.tsx ***!
  \***********************/
/*! no static exports found */
/***/ (function(module, exports, __webpack_require__) {

"use strict";

Object.defineProperty(exports, "__esModule", { value: true });
var React = __webpack_require__(/*! react */ "react");
var ReactDOM = __webpack_require__(/*! react-dom */ "react-dom");
var Root_1 = __webpack_require__(/*! ./components/Root */ "./src/components/Root.tsx");
window.modules = {};
if (!Array.prototype.find) {
    Object.defineProperty(Array.prototype, "find", {
        value: function (predicate) {
            // 1. Let O be ? ToObject(this value).
            if (this == null) {
                throw new TypeError('"this" is null or not defined');
            }
            var o = Object(this);
            // 2. Let len be ? ToLength(? Get(O, "length")).
            var len = o.length >>> 0;
            // 3. If IsCallable(predicate) is false, throw a TypeError exception.
            if (typeof predicate !== "function") {
                throw new TypeError("predicate must be a function");
            }
            // 4. If thisArg was supplied, let T be thisArg; else let T be undefined.
            var thisArg = arguments[1];
            // 5. Let k be 0.
            var k = 0;
            // 6. Repeat, while k < len
            while (k < len) {
                // a. Let Pk be ! ToString(k).
                // b. Let kValue be ? Get(O, Pk).
                // c. Let testResult be ToBoolean(? Call(predicate, T, « kValue, k, O »)).
                // d. If testResult is true, return kValue.
                var kValue = o[k];
                if (predicate.call(thisArg, kValue, k, o)) {
                    return kValue;
                }
                // e. Increase k by 1.
                k++;
            }
            // 7. Return undefined.
            return undefined;
        },
        configurable: true,
        writable: true
    });
}
if (!Array.prototype.findIndex) {
    Object.defineProperty(Array.prototype, "findIndex", {
        value: function (predicate) {
            // 1. Let O be ? ToObject(this value).
            if (this == null) {
                throw new TypeError('"this" is null or not defined');
            }
            var o = Object(this);
            // 2. Let len be ? ToLength(? Get(O, "length")).
            var len = o.length >>> 0;
            // 3. If IsCallable(predicate) is false, throw a TypeError exception.
            if (typeof predicate !== "function") {
                throw new TypeError("predicate must be a function");
            }
            // 4. If thisArg was supplied, let T be thisArg; else let T be undefined.
            var thisArg = arguments[1];
            // 5. Let k be 0.
            var k = 0;
            // 6. Repeat, while k < len
            while (k < len) {
                // a. Let Pk be ! ToString(k).
                // b. Let kValue be ? Get(O, Pk).
                // c. Let testResult be ToBoolean(? Call(predicate, T, « kValue, k, O »)).
                // d. If testResult is true, return k.
                var kValue = o[k];
                if (predicate.call(thisArg, kValue, k, o)) {
                    return k;
                }
                // e. Increase k by 1.
                k++;
            }
            // 7. Return -1.
            return -1;
        },
        configurable: true,
        writable: true
    });
}
ReactDOM.render(React.createElement(Root_1.Root, null), document.getElementById("example"));


/***/ }),

/***/ "react":
/*!************************!*\
  !*** external "React" ***!
  \************************/
/*! no static exports found */
/***/ (function(module, exports) {

module.exports = React;

/***/ }),

/***/ "react-dom":
/*!***************************!*\
  !*** external "ReactDOM" ***!
  \***************************/
/*! no static exports found */
/***/ (function(module, exports) {

module.exports = ReactDOM;

/***/ })

/******/ });
//# sourceMappingURL=bundle.js.map