use UIState;
use web_view::WebView;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::fmt;
use shared::ChannelUpdate;
use serde_json;

pub struct Client {
    receiver: Receiver<ClientMessage>,
}

impl Client {
    pub fn new(webview: &WebView<UIState>) -> ClientHandle {
        /*
        spawn(move || {
            sleep_ms(1000);
            let mut client = Rc::new(shared::client::Client::new("client", ClientState { webview }));
            client.register_listener("*", client_all_received);
            client.on_startup(|startup| {
                startup.emit.push(serde_json::from_str("{\"action\":\"node.list\"}").unwrap());
            });
            let client_clone = client.clone();
            webview.dispatch(move |_, state| {
                state.client = Some(client_clone);
            });
            client.launch();
        });
        */
        ClientHandle {
            sender
        }
    }
}

pub struct ClientHandle {
    sender: Sender<ClientMessage>,
}

impl fmt::Debug for ClientHandle {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        Ok(())
    }
}

impl ClientHandle {
    pub fn send(&mut self, value: &serde_json::Value) {
        unimplemented!();
    }
}

enum ClientMessage {

}

struct ClientState {
}

impl fmt::Debug for ClientState {
    fn fmt(&self, _fmt: &mut fmt::Formatter) -> fmt::Result {
        Ok(())
    }
}

fn client_all_received(update: &mut ChannelUpdate<ClientState>) {
    let str = format!(
        "external_message_received(\"{}\", {:?})",
        update.channel,
        serde_json::to_string(&update.value).unwrap()
    );
    /*
    update.state.webview.dispatch(move |webview, _| {
        println!("{}", str);
        webview.eval(&str);
    });
    */
}

