import * as React from "react";
import { ContainerComponent } from "./ContainerComponent";

export class Dashboard extends ContainerComponent {
    toggle_active(newstate: boolean): void {
        throw new Error('Method not implemented.');
    }

    render_title(): JSX.Element {
        return <span>
            Dashboard
        </span>;
    }

    render(): JSX.Element {
        return <div />;
    }

}