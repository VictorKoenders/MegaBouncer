// #![windows_subsystem = "windows"]
#![allow(deprecated)]

extern crate serde_json;
extern crate shared;
extern crate web_view;
extern crate hyper;

mod web_server;

use std::thread::spawn;
use web_view::*;

#[derive(Default, Debug)]
struct UIState {
    pub modules: Vec<Module>,
}

struct ClientState {
    webview: MyUnique<WebView<'static, UIState>>,
}

#[derive(Debug)]
pub struct Module {
    id: String,
    name: String,
    ui: Option<String>,
}

fn main() {
    let url = web_server::serve();
    println!("Requesting {:?}", url);
    let (_result_userdata, success) = run(
        "Megabouncer",
        Content::Url(url),
        Some((800, 600)),
        true,
        true,
        init_cb,
        invoke_cb,
        Default::default(),
    );
    println!("Success? {:?}", success);
}

fn client_all_received(update: &mut shared::ChannelUpdate<ClientState>) {
    let str = format!(
        "message_received(\"{}\", {:?})",
        update.channel,
        serde_json::to_string(&update.value).unwrap()
    );
    update.state.webview.dispatch(move |webview, _| {
        webview.eval(&str);
    });
}

fn init_cb(webview: MyUnique<WebView<'static, UIState>>) {
    spawn(move || {
        let mut client = shared::client::Client::new("client", ClientState { webview });
        client.register_listener("*", client_all_received);
        client.on_startup(|startup| {
            startup.emit.push(serde_json::from_str("{\"action\":\"node.list\"}").unwrap());
        });
        client.launch();
    });
}

fn invoke_cb(webview: &mut WebView<UIState>, arg: &str, _userdata: &mut UIState) {
    let mut iter = arg.split(':');
    match iter.next() {
        Some("exit") => {
            webview.terminate();
        }
        Some("keydown") => {
            if let Some(Ok(code)) = iter.next().map(|c| c.parse::<i32>()) {
                if code == 27 {
                    webview.terminate();
                }
            }
        }
        Some("log") => {
            let line = iter.collect::<Vec<_>>().join(", ");
            println!("{}", line);
        }
        _ => unimplemented!(),
    }
}
