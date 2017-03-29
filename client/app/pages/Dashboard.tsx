import * as React from "react";
import { ContainerComponent } from "./Container";

export class Dashboard implements ContainerComponent {
    render_title(): JSX.Element {
        return <span>
            Dashboard
        </span>;
    }
    title_changed: () => void;

    render(): JSX.Element {
        return <div />;
    }

}