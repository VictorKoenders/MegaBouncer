import * as React from "react";
import { Action, RootState, Module } from "../handler/base";

import NodeChannelRegistered from "../handler/node.channel.registered";
import NodeDisconnected from "../handler/node.disconnected";
import NodeIdentified from "../handler/node.identified";
import NodeListed from "../handler/node.listed";

import UIGotten from "../handler/ui.gotten";

const handlers: {
  [name: string]: (state: RootState, obj: any) => void;
} = {
  "node.channel.registered": NodeChannelRegistered,
  "node.disconnected": NodeDisconnected,
  "node.identified": NodeIdentified,
  "node.listed": NodeListed,
  "ui.gotten": UIGotten,
};

export class Root extends React.Component<{}, RootState> {
  constructor(props: {}, context?: any) {
    super(props, context);
    // window.message_received = this.message_received.bind(this);
    this.state = {
      logs: [],
      modules: [],
      active: null
    };
  }
  message_received(channel: string, obj: Action) {
    this.setState(oldstate => {
      if (handlers[channel]) {
        handlers[channel](oldstate, obj);
      } else {
        console.log("Channel without listener:");
        console.log(obj);
      }
      return oldstate;
    });
  }

  module_clicked(index: number | null, e: React.MouseEvent<HTMLAnchorElement>) {
    e.preventDefault();
    e.stopPropagation();

    if (index == null) {
      this.setState(oldState => ({
        active: null
      }));
    } else {
      this.setState(oldState => ({
        active: oldState.modules[index]
      }));
    }

    return false;
  }

  render_link(module: Module | null = null, index: number = -1) {
    let name = module === null ? "Home" : module.name;
    if (module !== null && !module.ui_loaded) return null;
    return (
      <li key={index} className={this.state.active == module ? "active" : ""}>
        <a href="#" onClick={this.module_clicked.bind(this, index)}>
          {name}
        </a>
      </li>
    );
  }
  render_content(): JSX.Element | null {
    if (this.state.active == null) return null;
    /*if (!window.modules.hasOwnProperty(this.state.active.name)) {
      return (
        <>
          <b>Module {this.state.active.name} is not found in the list:</b>
          <br />
          <ul>
            {Object.keys(window.modules).map((m, i) => <li key={i}>{m}</li>)}
          </ul>
        </>
      );
    }
    let Connector = window.modules[this.state.active.name];
    return <Connector />;*/
    return null;
  }
  render() {
    return (
      <>
        <ul className="top_bar">
          {this.render_link()}
          {this.state.modules.map(this.render_link.bind(this))}
        </ul>
        {this.render_content()}
      </>
    );
  }
}
