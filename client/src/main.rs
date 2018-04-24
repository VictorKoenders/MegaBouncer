// #![windows_subsystem = "windows"]
#![allow(deprecated)]

extern crate web_view;

use std::thread::{sleep_ms, spawn};
use web_view::*;

#[derive(Default, Debug)]
struct UserState {
	pub counter: i32,
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
                userdata.counter -= 1;
                render(webview, userdata);
            });
        }
        sleep_ms(1000);
    });
}

fn invoke_cb(webview: &mut WebView<UserState>, arg: &str, userdata: &mut UserState) {
    match arg {
        "reset" => {
            userdata.counter += 10;
            render(webview, userdata);
        }
        "exit" => {
            webview.terminate();
        }
		x if x.starts_with("keydown:") => {
			let code = x.split(':').nth(1);
			if let Some(Ok(code)) = code.map(|c| c.parse::<i32>()) {
				println!("Key code {:?}", code);
				if code == 27 {
					webview.terminate();
				}
			}
		}
		x if x.starts_with("log:") => {
			let line = x.split(':').skip(1).collect::<Vec<_>>().join(", ");
			println!("{}", line);
		}
        _ => unimplemented!(),
    }
}

fn render(webview: &mut WebView<UserState>, userdata: &UserState) {
    webview.eval(&format!("updateTicks({})", userdata.counter));
}

const HTML: &'static str = r#"
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
