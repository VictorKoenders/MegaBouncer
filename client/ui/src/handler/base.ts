export interface Action {
  sender_id: string;
  sender: string;
  action: string;
}

export interface RootState {
  logs: Array<{ channel: string; obj: Action }>;
  modules: Module[];
  active: Module | null;
}

export interface Module {
  id: string;
  name: string;
  ui_loaded: boolean;
}


