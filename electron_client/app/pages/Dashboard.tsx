import * as React from "react";
import { ContainerComponent } from "./ContainerComponent";

export class Dashboard extends ContainerComponent {

    render_title(): JSX.Element {
        return <span>
            Dashboard
        </span>;
    }

    render(): JSX.Element {
        return <div />;
    }

}