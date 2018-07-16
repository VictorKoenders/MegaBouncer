// #![windows_subsystem = "windows"]
#![allow(deprecated)]

extern crate hyper;
extern crate serde_json;
extern crate shared;
extern crate web_view;

mod web_server;
// mod client;

use shared::client::ClientHandle;
use shared::ChannelUpdate;
use std::thread::{sleep_ms, spawn};
use web_view::*;

#[derive(Default)]
pub struct UIState {
    handle: Option<ClientHandle>,
}

pub struct ClientState {
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

fn init_cb(webview: MyUnique<WebView<'static, UIState>>) {
    spawn(move || {
        sleep_ms(1000);
        let mut client = shared::client::Client::new(
            "client",
            ClientState {
                webview: webview.clone(),
            },
        );
        client.register_listener("*", client_all_received);
        client.on_startup(|startup| {
            startup
                .emit
                .push(serde_json::from_str("{\"action\":\"node.list\"}").unwrap());
        });
        let handle = client.launch_async();

        webview.dispatch(move |_, state| {
            let handle = handle.clone();
            state.handle = Some(handle);
        });
    });
}

fn client_all_received(update: &mut ChannelUpdate<ClientState>) {
    let str = format!(
        "external_message_received(\"{}\", {:?})",
        update.channel,
        serde_json::to_string(&update.value).unwrap()
    );
    update.state.webview.dispatch(move |webview, _| {
        webview.eval(&str);
    });
}

fn invoke_cb(webview: &mut WebView<UIState>, arg: &str, userdata: &mut UIState) {
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
            let line = iter.collect::<Vec<_>>().join(":");
            println!("{}", line);
        }
        Some("emit") => {
            let remaining = iter.collect::<Vec<_>>().join(":");
            let value: serde_json::Value = serde_json::from_str(&remaining).unwrap();
            println!("Sending {:?}", value);
            userdata.handle.as_mut().unwrap().send(value);
        }
        _ => unimplemented!(),
    }
}
