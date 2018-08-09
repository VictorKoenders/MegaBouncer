import { Action, RootState } from "./base";

export interface NodeDisconnectedAction extends Action {
    id: string;
    name: string;
}

export default function(state: RootState, obj: NodeDisconnectedAction) {
    let index = state.modules.findIndex(m => m.id == obj.id);
    if(index >= 0) {
        /*if(window.modules.hasOwnProperty(state.modules[index].name)) {
            delete window.modules[state.modules[index].name];
        }*/
        state.modules.splice(index, 1);
    }
}
