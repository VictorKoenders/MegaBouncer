#[macro_use]
extern crate yew;
use yew::prelude::*;

struct AppContext;

struct AppComponent;

enum AppMessage {
}

impl Component<AppContext> for AppComponent {
    type Message = AppMessage;
    type Properties = ();

    fn create(_props: (), _context: &mut Env<AppContext, Self>) -> Self {
        AppComponent
    }

    fn update(&mut self, _msg: AppMessage, _context: &mut Env<AppContext, Self>) -> ShouldRender {
        true
    }
}

impl Renderable<AppContext, AppComponent> for AppComponent {
    fn view(&self) -> Html<AppContext, AppComponent> {
        html! {
            <h2>{"Hello from AppComponent"}</h2>
        }
    }
}

fn main() {
    yew::initialize();
    let app = App::<_, AppComponent>::new(AppContext);
    app.mount_to_body();
    yew::run_loop();
}

