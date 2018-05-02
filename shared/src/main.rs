extern crate shared;

pub fn main(){
    let mut client = shared::client::Client::new("Test", ());
    client.register_listener("*", catch_all);
    client.launch();
}

fn catch_all(_: &mut (), name: &str, value: &shared::serde_json::Value) {
    println!("{:?} {:?}", name, value);
}
