import * as React from "react";

export interface RootState {
  logs: Array<{ channel: string; obj: Action }>;
  modules: Module[];
}

export interface Module {
  id: string;
  name: string;
  has_ui?: boolean;
}

class Action {
  action: string;
}

class NodeListAction extends Action {
  nodes: NodeListAction_Node[];
}

class NodeListAction_Node {
  channels: string[];
  id: string;
  name: string;
}

export class Root extends React.Component<{}, RootState> {
  constructor(props: {}, context?: any) {
    super(props, context);
    window.message_received = this.message_received.bind(this);
    this.state = {
      logs: [],
      modules: []
    };
  }
  message_received(channel: string, obj: Action) {
    this.setState(oldstate => {
      let new_state = {
        logs: [{ channel, obj }, ...oldstate.logs],
        modules: [...oldstate.modules]
      };
      if (channel == "node.listed") {
        for (const node of (obj as NodeListAction).nodes) {
          new_state.modules.push({
            id: node.id,
            name: node.name
          });
          if (node.channels.indexOf("get_ui") != -1) {
            let emit =
              "emit:" +
              JSON.stringify({
                action: "get_ui",
                target: node.id
              });
            console.log(emit);
            (external as any).invoke(emit);
          }
        }
      }
      return new_state;
    });
  }
  render() {
    return (
      <ul>
        {this.state.logs.map((log, index) => (
          <li key={index}>
            <b>{log.channel}</b>:<br />
            {JSON.stringify(log.obj)}
          </li>
        ))}
      </ul>
    );
  }
}
