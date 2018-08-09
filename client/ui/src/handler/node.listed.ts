import { RootState, Action } from "./base";

interface NodeListAction extends Action {
  nodes: NodeListAction_Node[];
}

interface NodeListAction_Node {
  channels: string[];
  id: string;
  name: string;
}

export function get_emit(id: string) {
  return (
    "emit:" +
    JSON.stringify({
      action: "ui.get",
      target: id
    })
  );
}

export default function(new_state: RootState, obj: NodeListAction) {
  for (const node of (obj as NodeListAction).nodes) {
    if (node.channels.indexOf("ui.get") != -1) {
      new_state.modules.push({
        id: node.id,
        name: node.name,
        ui_loaded: false
      });
      let emit = get_emit(node.id);
      console.log("Emitting", emit);
      //external.invoke(emit);
    }
  }
}
