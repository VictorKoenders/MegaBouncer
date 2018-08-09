import { RootState, Action } from "./base";

export interface UIAction extends Action {
  ui: string;
}

export default function(new_state: RootState, obj: UIAction) {
  let module = new_state.modules.find(m => m.id == obj.sender_id);
  try {
    eval(obj.ui);
    module.ui_loaded = true;
  } catch (e) {
    console.log(e);
  }
}
