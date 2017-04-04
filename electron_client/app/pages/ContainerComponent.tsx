export abstract class ContainerComponent {
    constructor(){}
    title_changed: () => void;
    state_changed: () => void;
    active: boolean;

    abstract render_title(): JSX.Element;
    abstract render(): JSX.Element;
    toggle_active(newstate: boolean) {
        this.active = newstate;
    }
}
