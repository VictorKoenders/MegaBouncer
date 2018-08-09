import { Action, RootState } from "./base";

export interface NodeIdentifiedAction extends Action {
    id: string;
    name: string;
}

export default function(state: RootState, obj: NodeIdentifiedAction) {
    let module = state.modules.find(m => m.id == obj.id);
    if(module) return;
    module = {
        id: obj.id,
        name: obj.name,
        ui_loaded: false,
    };
    state.modules.push(module);
}
