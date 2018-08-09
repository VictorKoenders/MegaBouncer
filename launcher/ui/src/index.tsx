import * as React from "react";
import * as ReactDOM from "react-dom";

import { Root } from "./components/Root";

declare global {
  interface Array<T> {
    find(cb: (t: T, i: number, arr: Array<T>) => boolean): T | null;
    findIndex(cb: (t: T, i: number, arr: Array<T>) => boolean): number | null;
  }
}

import "./polyfill";
ReactDOM.render(<Root />, document.getElementById("root"));
