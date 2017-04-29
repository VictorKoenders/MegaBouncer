import * as React from "react";
import { ContainerComponent } from "./ContainerComponent";

interface ClientMessage {
    clients: Array<string>;
}
class Node {
    name: string;
    sendQueue: Array<any>;
}
class NodesState {
    nodes: Array<Node>;
}

export class Nodes extends ContainerComponent {
    state: NodesState;

    constructor() {
        super();
        this.state = {
            nodes: []
        };
        this.register_listener("server.clients", this.clients_received);
        this.register_listener("server.client.connected", this.client_connected);
        this.register_listener("server.client.disconnected", this.client_disconnected);
        this.register_listener("*", this.any_received);
        this._connector.send_raw("get_clients", "", {});
    }

    client_connected(message: any){
        console.log('connected', JSON.stringify(message));
    }

    client_disconnected(message: any){
        var index = this.state.nodes.findIndex(n => n.name == message.name);
        if(index != -1){
            this.state.nodes.splice(index, 1);
            this.state_changed();
            this.title_changed();
        }
    }

    any_received(message: any){
        console.log(JSON.stringify(message));
        if(message.sender){
            var node = this.state.nodes.find(n => n.name == message.sender);
            if(node == null){
                console.log('creating new node for ', message.sender);
                node = { name: message.sender, sendQueue: [] };
                this.state.nodes.push(node);
                this.title_changed();
            }
            node.sendQueue.unshift(message);
            while(node.sendQueue.length > 10){
                node.sendQueue.pop();
            }
            this.state_changed();
        }
    }

    clients_received(clients: ClientMessage) {
        for(let name of clients.clients){
            let client = this.state.nodes.find(n => n.name == name);
            if(client == null){
                this.state.nodes.push({
                    name: name,
                    sendQueue: []
                });
            }
        }

        for(let client of this.state.nodes.filter(c => clients.clients.every(cl => cl != c.name))) {
            this.state.nodes.splice(this.state.nodes.indexOf(client), 1);
        }
        this.title_changed();
        this.state_changed();
    }
    render_title() {
        return <span>
            Nodes ({this.state.nodes.length})
        </span>;
    }

    render_node(node: Node, index: number): JSX.Element {
        return <li key={index}>
            <b>{node.name} ({node.sendQueue.length})</b><br />
            <ul>
                {node.sendQueue.map((item, index) => 
                    <li key={index}>{JSON.stringify(item)}</li>
                )}
            </ul>
        </li>
    }
    render() {
        return <ul>
            {this.state.nodes.map(this.render_node.bind(this))}
        </ul>
    }

}