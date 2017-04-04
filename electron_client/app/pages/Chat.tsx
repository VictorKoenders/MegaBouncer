import * as React from "react";
import { ContainerComponent } from "./ContainerComponent";

class ChatState {
    count: number;
    scrollback: Array<ChatMessage>;
    input_text: string;
}

class ChatMessage {
    sender: string;
    message: string;
}

declare var remote: {
    getGlobal: (name: string) => any,
};

export class Chat extends ContainerComponent {
    connector: any;
    state: ChatState;

    constructor() {
        super();
        this.state = {
            count: 0,
            scrollback: [],
            input_text: ''
        };
        this.connector = remote.getGlobal('connector');
        this.connector.on('irc.message', this.irc_message_received.bind(this));
    }

    toggle_active(newstate: boolean) {
        if(newstate){
            this.state.count = 0;
            this.state_changed();
        }
        super.toggle_active(newstate);
    }

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

    irc_message_received(data: any){
        if(data.message.type == "privmsg") {
            console.log(JSON.stringify(data));
            var message = {
                sender: data.message.sender.name,
                message: data.message.message
            };
            if(message.sender[0] == ':') message.sender = message.sender.substring(1);
            this.state.scrollback.push(message);
            if(data.message.message.indexOf('Trangar') != -1){
                this.state.count ++;
                this.title_changed();
            }
            this.state_changed();
        }
    }

    tick(){
        this.state.count += 1;
        this.title_changed();
    }

    send_text(e: Event) {
        e.preventDefault();
        this.connector.send_emit('irc.send', {
            host: 'irc.esper.net',
            port: 6667,
            type: 'privmsg',
            target: 'Trangar',
            message: this.state.input_text
        });
        this.state.input_text = '';
        this.state_changed();
    }

    update_text(e: Event){
        this.state.input_text = (e.target as HTMLInputElement).value;
        this.state_changed();
    }

    render(): JSX.Element {
        return <div>
            {this.state.scrollback.map((msg, index) => 
                <div key={index}>
                    <b>{msg.sender}: </b>
                    {msg.message}
                </div>
            )}
            <div className="float-bottom">
                <form onSubmit={this.send_text.bind(this)}>
                    <input type="text" value={this.state.input_text} onChange={this.update_text.bind(this)} />
                </form>
            </div>
        </div>;
    }
}