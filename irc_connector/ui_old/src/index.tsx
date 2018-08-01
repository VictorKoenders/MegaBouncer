import * as React from "react";
import * as ReactDOM from "react-dom";

class Root extends React.Component<{}, {}> {
    render(){
        return <h2>Hello from IRC Connector</h2>;
    }
}

window.modules["Irc connector"] = Root;

declare global {
    interface Window {
        modules: {[name:string]: any}
    }
}