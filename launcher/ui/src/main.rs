#[macro_use]
extern crate yew;
extern crate failure;
extern crate serde_json;
#[macro_use]
extern crate stdweb;

use yew::prelude::*;
use yew::services::websocket::{WebSocketService, WebSocketTask};
use yew::services::console::ConsoleService;

type Result<T> = std::result::Result<T, failure::Error>;

struct AppContext {
    ws: WebSocketService,
    console: ConsoleService,
}

struct AppComponent {
    modules: Vec<Module>,
    ws: WebSocketTask,
}

#[derive(Debug)]
struct Module {
    name: String,
    status: ModuleStatus,
}

#[derive(Debug)]
enum ModuleStatus {
    Building,
    Failed,
    Running,
}

impl ModuleStatus {
    pub fn class_name(&self) -> &'static str {
        match self {
            ModuleStatus::Building => "info",
            ModuleStatus::Failed => "error",
            ModuleStatus::Running => "success",
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            ModuleStatus::Building => "Building...",
            ModuleStatus::Failed => "Failed!",
            ModuleStatus::Running => "Running",
        }
    }
}

#[derive(Debug)]
enum AppMessage {
    Ws(WebSocket),
}

#[derive(Debug)]
enum WebSocket {
    Status(yew::services::websocket::WebSocketStatus),
    Message(StringWrapper),
}

#[derive(Debug)]
struct StringWrapper(Result<String>);

impl std::ops::Deref for StringWrapper {
    type Target = Result<String>;

    fn deref(&self) -> &Result<String>{
        &self.0
    }
}

impl From<yew::format::Text> for StringWrapper {
    fn from(f: yew::format::Text) -> StringWrapper {
        StringWrapper(f)
    }
}

impl From<yew::format::Binary> for StringWrapper {
    fn from(f: yew::format::Binary) -> StringWrapper {
        StringWrapper(f.and_then(|s| String::from_utf8(s).map_err(Into::into)))
    }
}

impl Component<AppContext> for AppComponent {
    type Message = AppMessage;
    type Properties = ();

    fn create(_props: (), context: &mut Env<AppContext, Self>) -> Self {
        let status_changed = context.send_back(|status| AppMessage::Ws(WebSocket::Status(status)));
        let message = context.send_back(|message| AppMessage::Ws(WebSocket::Message(message)));
        let url: String = js! {
            return "ws://localhost:" + document.location.port + "/ws/";
        }.into_string().unwrap();
        let ws = context.ws.connect(&url, message, status_changed);
        AppComponent {
            modules: Vec::new(),
            ws,
        }
    }

    fn update(&mut self, msg: AppMessage, context: &mut Env<AppContext, Self>) -> ShouldRender {
        context.console.log(&format!("{:?}", msg));
        true
    }
}

impl Renderable<AppContext, AppComponent> for AppComponent {
    fn view(&self) -> Html<AppContext, AppComponent> {
        html! {
            <>
            <h2>{"Status of modules"}</h2>
            <ul>
            {for self.modules.iter().map(|m| html! {
                <li>
                    <b>{&m.name}</b> <span class=("status", m.status.class_name()), >{m.status.name()}</span>
                </li>
            })}
            </ul>
            </>
        }
    }
}

fn main() {
    yew::initialize();
    let app = App::<_, AppComponent>::new(AppContext {
        ws: WebSocketService::new(),
        console: ConsoleService::new(),
    });
    app.mount_to_body();
    yew::run_loop();
}

pub struct State {

}

#[no_mangle]
pub extern "C" fn test(state: &mut State) {
    println!("Hello from test");
}
