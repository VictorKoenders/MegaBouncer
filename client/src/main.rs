// #![windows_subsystem = "windows"]
#![allow(deprecated)]

extern crate web_view;
extern crate shared;

use std::thread::{sleep_ms, spawn};
use web_view::*;

#[derive(Default, Debug)]
struct UserState {
    pub modules: Vec<Module>,
}

#[derive(Debug)]
pub struct Module {
    id: String,
    name: String,
    ui: Option<String>,
}

fn main() {
    let (result_userdata, success) = run(
        "Megabouncer",
        Content::Html(HTML),
        Some((800, 600)),
        true,
        true,
        init_cb,
        invoke_cb,
        Default::default(),
    );
    println!("Success? {:?}", success);
    println!("Last user data: {:?}", result_userdata);
}

fn init_cb(webview: MyUnique<WebView<'static, UserState>>) {
    spawn(move || loop {
        {
            webview.dispatch(|webview, userdata| {
            });
        }
        sleep_ms(1000);
    });
}

fn invoke_cb(webview: &mut WebView<UserState>, arg: &str, userdata: &mut UserState) {
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

const HTML: &str = r#"
<!doctype html>
<html>
	<body>
		<p id="ticks"></p>
		<button onclick="external.invoke('reset')">reset</button>
		<button onclick="external.invoke('exit')">exit</button>
		<script type="text/javascript">
			function updateTicks(u) {
				document.getElementById('ticks').innerHTML = 'userdata ' + u;
			}
			console.log = function(){
				var args = "log:";
				for(var i = 0; i < arguments.length; i++) {
					args += arguments[i];
				}
				external.invoke(args);
			}
			document.onkeydown = function(e){
				external.invoke("keydown:" + e.keyCode);
			}
		</script>
	</body>
</html>
"#;
