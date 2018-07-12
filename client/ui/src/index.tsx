import * as React from "react";
import * as ReactDOM from "react-dom";

import { Root } from "./components/Root";

ReactDOM.render(
    <Root />,
    document.getElementById("example")
);

declare global {
    interface Window {
        message_received: (message: string, value: any) => void;
    }
    interface External {
        invoke: (n: string) => void;
    }
}
