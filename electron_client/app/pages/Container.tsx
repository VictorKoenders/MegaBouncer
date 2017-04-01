import * as React from "react";
import { Dashboard } from "./Dashboard";
import { Chat } from "./Chat";
import { ContainerComponent } from "./ContainerComponent";

class ContainerState {
    components: Array<ContainerComponent>;
    active_index: number;
}

export class Container extends React.Component<{}, ContainerState> {
    constructor(){
        super();
        this.state = {
            components: Array<ContainerComponent>(
                new Dashboard(),
                new Chat()
            ),
            active_index: 0
        };

        this.state.components.forEach(component => {
            component.title_changed = this.component_title_changed.bind(this, component);
            component.state_changed = this.component_state_changed.bind(this, component);
        });
    }
    component_title_changed(component: ContainerComponent) {
        this.forceUpdate();
    }
    component_state_changed(component: ContainerComponent) {
        let index = this.state.components.indexOf(component);
        if(index == this.state.active_index){
            this.forceUpdate();
        }
    }
    component_clicked(component: ContainerComponent, index: number, event: Event){
        this.setState((current) => ({
            ...current,
            active_index: index
        }));
        console.log('selecting', component);
    }
    renderComponent(component: ContainerComponent, index: number) {
        const className = index == this.state.active_index ? "active" : "";
        return <li key={index} className={className} onClick={this.component_clicked.bind(this, component, index)}>
            <a href="#">{
                component.render_title()
            }</a>
        </li>;
    }
    render() {
        return <div className="container-fluid">
            <ul className="nav nav-tabs">
                {this.state.components.map(this.renderComponent.bind(this))}
            </ul>
            {this.state.components[this.state.active_index].render()}
        </div>;
    }
}
