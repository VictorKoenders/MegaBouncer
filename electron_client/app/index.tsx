import * as React from "react";
import * as ReactDOM from "react-dom";
import { Container } from "./pages/Container";

const element = document.createElement("div");
ReactDOM.render(
    <Container />,
    element
);
document.body.appendChild(element);

window.onkeydown = function(ev){
    if(ev.key == "F5"){
        document.location.reload();
    }
}
