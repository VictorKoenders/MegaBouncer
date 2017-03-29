import * as React from "react";
import { ContainerComponent } from "./Container";

class ChatState {
    count: number;
}

declare var remote: {
    getGlobal: (name: string) => any,
};

export class Chat implements ContainerComponent {
    render_title(): JSX.Element {
        return <span>
            Chat 
            {this.state.count ? " " : ""}
            {this.state.count ?  
            <span className="badge">
                {this.state.count}
            </span>
            : ""}
        </span>;
    }
    title_changed: () => void;

    state: ChatState;

    constructor() {
        this.state = {
            count: 0
        };
        const connector = remote.getGlobal('connector');
        connector.on('irc.message', this.irc_message_received.bind(this));
    }

    irc_message_received(data: any){
        console.log('got irc message', data);
    }

    tick(){
        this.state.count += 1;
        this.title_changed();
    }

    render(): JSX.Element {
        return <div />;
    }
}