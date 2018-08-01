import { Action, RootState } from "./base";
import { get_emit as request_ui_emit } from "./node.listed"

export interface ChannelRegisteredAction extends Action {
    channel: string;
}

export default function(state: RootState, obj: ChannelRegisteredAction) {
    const module = state.modules.find(m => m.id == obj.sender_id);
    if(!module) return;
    if(obj.channel == "ui.get") {
        external.invoke(request_ui_emit(obj.sender_id));
    }
}
