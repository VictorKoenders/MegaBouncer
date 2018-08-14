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
var __assign = (this && this.__assign) || Object.assign || function(t) {
    for (var s, i = 1, n = arguments.length; i < n; i++) {
        s = arguments[i];
        for (var p in s) if (Object.prototype.hasOwnProperty.call(s, p))
            t[p] = s[p];
    }
    return t;
};
Object.defineProperty(exports, "__esModule", { value: true });
var React = __webpack_require__(/*! react */ "react");
var Root = /** @class */ (function (_super) {
    __extends(Root, _super);
    function Root(props, context) {
        var _this = _super.call(this, props, context) || this;
        var socket = _this.start_websocket();
        _this.state = {
            running_processes: [],
            running_builds: [],
            finished_builds: [],
            projects: [],
            errors: [],
            open_uuids: [],
            socket: socket,
        };
        return _this;
    }
    Root.prototype.start_websocket = function () {
        var socket = new WebSocket("ws://" + document.location.host + "/ws");
        socket.onclose = this.ws_close.bind(this);
        socket.onopen = this.ws_open.bind(this);
        socket.onmessage = this.ws_message.bind(this);
        socket.onerror = this.ws_error.bind(this);
        return socket;
    };
    Root.prototype.ws_close = function (ev) {
        var _this = this;
        console.log("Websocket closed, reconnecting in 5 secs", ev);
        this.setState({ socket: null });
        setTimeout(function () {
            if (_this.state.socket === null) {
                var socket = _this.start_websocket();
                _this.setState({ socket: socket });
            }
        }, 5000);
    };
    Root.prototype.ws_error = function (ev) {
        console.log("Websocket error", ev);
    };
    Root.prototype.ws_open = function (ev) {
        console.log("Websocket opened");
    };
    Root.prototype.ws_message = function (ev) {
        var json = JSON.parse(ev.data);
        if (Array.isArray(json.running_processes)) {
            this.setState(json);
            return;
        }
        var change = json;
        if (change.ErrorAdded) {
            var errors = this.state.errors;
            errors.splice(0, 0, change.ErrorAdded);
            this.setState({ errors: errors });
        }
        else if (change.ProjectsSet) {
            this.setState({
                projects: change.ProjectsSet,
            });
            // Processes
        }
        else if (change.RunningProcessAdded) {
            var running_processes = this.state.running_processes;
            running_processes.push(change.RunningProcessAdded);
            this.setState({ running_processes: running_processes });
        }
        else if (change.RunningProcessRemoved) {
            var running_processes = this.state.running_processes;
            var index = running_processes.findIndex(function (p) { return p.uuid == change.RunningProcessRemoved; });
            if (index !== null) {
                running_processes.splice(index, 1);
                this.setState({ running_processes: running_processes });
            }
        }
        else if (change.RunningProcessStdout) {
            var running_processes = this.state.running_processes;
            var index = running_processes.findIndex(function (p) { return p.uuid == change.RunningProcessStdout[0]; });
            if (index !== null) {
                running_processes[index].stdout += change.RunningProcessStdout[1];
                this.setState({ running_processes: running_processes });
            }
        }
        else if (change.RunningProcessStderr) {
            var running_processes = this.state.running_processes;
            var index = running_processes.findIndex(function (p) { return p.uuid == change.RunningProcessStderr[0]; });
            if (index !== null) {
                running_processes[index].stderr += change.RunningProcessStderr[1];
                this.setState({ running_processes: running_processes });
            }
        }
        else if (change.RunningProcessTerminated) {
            var running_processes = this.state.running_processes;
            var index = running_processes.findIndex(function (p) { return p.uuid == change.RunningProcessTerminated[0]; });
            if (index !== null) {
                running_processes.splice(index, 1);
                this.setState({ running_processes: running_processes });
            }
        }
        else if (change.RunningProcessFinished) {
            var running_processes = this.state.running_processes;
            var index = running_processes.findIndex(function (p) { return p.uuid == change.RunningProcessFinished[0]; });
            if (index !== null) {
                running_processes.splice(index, 1);
                this.setState({ running_processes: running_processes });
            }
            // Builds
        }
        else if (change.RunningBuildAdded) {
            var running_builds = this.state.running_builds;
            running_builds.push(change.RunningBuildAdded);
            this.setState({ running_builds: running_builds });
        }
        else if (change.RunningBuildStdout) {
            var running_builds = this.state.running_builds;
            var index = running_builds.findIndex(function (b) { return b.uuid == change.RunningBuildStdout[0]; });
            if (index !== null) {
                running_builds[index].stdout += change.RunningBuildStdout[1];
                this.setState({ running_builds: running_builds });
            }
        }
        else if (change.RunningBuildStderr) {
            var running_builds = this.state.running_builds;
            var index = running_builds.findIndex(function (b) { return b.uuid == change.RunningBuildStderr[0]; });
            if (index !== null) {
                running_builds[index].stderr += change.RunningBuildStderr[1];
                this.setState({ running_builds: running_builds });
            }
        }
        else if (change.RunningBuildTerminated) {
            var running_builds = this.state.running_builds;
            var index = running_builds.findIndex(function (b) { return b.uuid == change.RunningBuildTerminated[0]; });
            if (index !== null) {
                var running_build = running_builds.splice(index, 1)[0];
                var finished_builds = this.state.finished_builds;
                var finished_build = __assign({ ended_on: new Date().toISOString(), status: -1, error: change.RunningBuildTerminated[1] }, running_build);
                finished_builds.splice(0, 0, finished_build);
                this.setState({ running_builds: running_builds, finished_builds: finished_builds });
            }
        }
        else if (change.RunningBuildFinished) {
            var running_builds = this.state.running_builds;
            var index = running_builds.findIndex(function (b) { return b.uuid == change.RunningBuildFinished[0]; });
            if (index !== null) {
                var running_build = running_builds.splice(index, 1)[0];
                var finished_builds = this.state.finished_builds;
                var finished_build = __assign({ ended_on: new Date().toISOString(), status: change.RunningBuildFinished[1], error: null }, running_build);
                finished_builds.splice(0, 0, finished_build);
                this.setState({ running_builds: running_builds, finished_builds: finished_builds });
            }
        }
        else {
            console.log("Unknown server command", change);
        }
    };
    Root.prototype.render_time = function (diff) {
        diff = Math.ceil(diff / 1000);
        var result = "";
        var show_seconds = true;
        var show_minutes = true;
        var show_hours = true;
        if (diff >= 86400) {
            var days = Math.floor(diff / 86400);
            diff -= days * 3600;
            result += days + " days";
            show_minutes = false;
            show_seconds = false;
        }
        if (diff >= 3600 && show_hours) {
            var hours = Math.floor(diff / 3600);
            diff -= hours * 3600;
            result += hours + " hours";
            show_seconds = false;
        }
        if (diff >= 60 && show_minutes) {
            if (result)
                result += ", ";
            var minutes = Math.floor(diff / 60);
            diff -= minutes * 60;
            result += minutes + " minutes";
        }
        if (diff > 0 && show_seconds) {
            if (result)
                result += ", ";
            result += diff + " seconds";
        }
        return result;
    };
    Root.prototype.render_running_build = function (build, index) {
        var start = new Date(build.started_on);
        var diff = Date.now() - start.getTime();
        return React.createElement("div", { key: index },
            React.createElement("p", { onClick: this.toggle_open.bind(this, build.uuid) },
                React.createElement("b", null,
                    build.directory,
                    "::",
                    build.build),
                " (running for ",
                this.render_time(diff),
                ")"),
            React.createElement("pre", null, build.stdout),
            React.createElement("pre", null, build.stderr));
    };
    Root.prototype.render_finished_build = function (build, index) {
        var start = new Date(build.started_on);
        var end = new Date(build.ended_on);
        var diff = end.getTime() - start.getTime();
        var is_open = this.state.open_uuids.some(function (u) { return u == build.uuid; });
        var status_text, status_color;
        if (build.error || build.status !== 0) {
            status_text = build.error || "Error";
            status_color = "red";
        }
        else {
            status_text = "Success";
            status_color = "green";
        }
        var title = React.createElement("p", { onClick: this.toggle_open.bind(this, build.uuid), key: index },
            React.createElement("b", null,
                build.directory,
                "::",
                build.build),
            ' ',
            React.createElement("b", { style: { color: status_color } }, status_text),
            ' ',
            "(finished ",
            this.render_time(Date.now() - end.getTime()),
            " ago, in ",
            this.render_time(diff),
            ")");
        if (!is_open) {
            return title;
        }
        return React.createElement("div", { key: index },
            title,
            React.createElement("pre", null, build.stdout),
            React.createElement("pre", null, build.stderr));
    };
    Root.prototype.render_process = function (process, index) {
        var is_open = this.state.open_uuids.some(function (u) { return u == process.uuid; });
        if (is_open) {
            return React.createElement("div", { key: index },
                React.createElement("p", { onClick: this.toggle_open.bind(this, process.uuid) },
                    React.createElement("b", null, process.directory),
                    " ",
                    React.createElement("a", { href: "#", onClick: this.kill_process.bind(this, process.pid) }, "\u00D7")),
                React.createElement("pre", null, process.stdout),
                React.createElement("pre", null, process.stderr));
        }
        else {
            return React.createElement("p", { key: index, onClick: this.toggle_open.bind(this, process.uuid) },
                React.createElement("b", null, process.directory),
                " ",
                React.createElement("a", { href: "#", onClick: this.kill_process.bind(this, process.pid) }, "\u00D7"));
        }
    };
    Root.prototype.kill_process = function (id, ev) {
        ev.preventDefault();
        ev.stopPropagation();
        if (this.state.socket) {
            console.log(JSON.stringify({
                kill: id
            }));
            this.state.socket.send(JSON.stringify({
                kill: id
            }));
        }
        return false;
    };
    Root.prototype.render_project = function (project, index) {
        return React.createElement("p", { key: index },
            React.createElement("b", null, project.name),
            " ",
            project.builds.map(this.render_build.bind(this, project)));
    };
    Root.prototype.render_build = function (project, build, index) {
        return React.createElement("button", { key: index, onClick: this.start_build.bind(this, project, build) }, build.name);
    };
    Root.prototype.toggle_open = function (uuid, ev) {
        ev.preventDefault();
        ev.stopPropagation();
        var uuids = this.state.open_uuids;
        var index = uuids.findIndex(function (u) { return u == uuid; });
        if (index !== null && index >= 0) {
            uuids.splice(index, 1);
        }
        else {
            uuids.push(uuid);
        }
        this.setState({
            open_uuids: uuids,
        });
        return false;
    };
    Root.prototype.start_build = function (project, build, ev) {
        ev.preventDefault();
        ev.stopPropagation();
        if (this.state.socket) {
            this.state.socket.send(JSON.stringify({
                start: [project.name, build.name]
            }));
        }
        return false;
    };
    Root.prototype.render_error = function (err, index) {
        return React.createElement("p", { key: index },
            React.createElement("b", null,
                this.render_time(Date.now() - new Date(err.time).getTime()),
                " ago"),
            React.createElement("br", null),
            err.error);
    };
    Root.prototype.render = function () {
        return React.createElement(React.Fragment, null,
            React.createElement("div", { style: { flex: 1, display: 'flex', flexDirection: 'row', borderBottom: '1px solid #555' } },
                React.createElement("div", { style: { flex: 1, overflow: "auto", borderRight: '1px solid #555', padding: 5 } },
                    React.createElement("h2", null, "Processes:"),
                    this.state.running_processes.map(this.render_process.bind(this))),
                React.createElement("div", { style: { flex: 1, overflow: "auto", borderRight: '1px solid #555', padding: 5 } }, this.state.errors.map(this.render_error.bind(this))),
                React.createElement("div", { style: { flex: 1, overflow: "auto", padding: 5 } }, this.state.projects.map(this.render_project.bind(this)))),
            React.createElement("div", { style: { flex: 1, display: 'flex', flexDirection: 'row' } },
                React.createElement("div", { style: { flex: 1, overflow: "auto", borderRight: '1px solid #555', padding: 5 } },
                    React.createElement("h2", null, "Running:"),
                    this.state.running_builds.map(this.render_running_build.bind(this))),
                React.createElement("div", { style: { flex: 1, overflow: "auto", padding: 5 } },
                    React.createElement("h2", null, "Finished:"),
                    this.state.finished_builds.map(this.render_finished_build.bind(this)))));
    };
    return Root;
}(React.Component));
exports.Root = Root;


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
__webpack_require__(/*! ./polyfill */ "./src/polyfill.ts");
ReactDOM.render(React.createElement(Root_1.Root, null), document.getElementById("root"));


/***/ }),

/***/ "./src/polyfill.ts":
/*!*************************!*\
  !*** ./src/polyfill.ts ***!
  \*************************/
/*! no static exports found */
/***/ (function(module, exports, __webpack_require__) {

"use strict";

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