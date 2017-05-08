extern crate serde_json;
extern crate shared;
extern crate redis;
extern crate config;

use redis::{Client, Commands, Connection};
use shared::prelude::*;

fn main() {
    let mut client = shared::Client::default();
    client.register::<RedisConnector>();
    client.start("datastore");
}

struct RedisConnector {
    connection: Connection
}

impl Default for RedisConnector {
    fn default() -> RedisConnector {
        let mut config = config::Config::new();
        let config_file = config::File::from_str(include_str!("../config.toml"), config::FileFormat::Toml).required(true);
        config.merge(config_file).expect("Loading config.toml");
        let host = config.get("redis-url").expect("Getting redis url").into_str().expect("Unwrapping redis url as string");
        let host: &str = &host;

        let client = Client::open(host).expect("Connection to redis client");
        let con: Connection = client.get_connection().expect("Opening redis connection");

        RedisConnector {
            connection: con
        }
    }
}

impl Component for RedisConnector {
    fn init(&self, _poll: &Poll) -> Vec<ComponentResponse>{
        vec![
            ComponentResponse::ListenToChannel(Channel::from_string("data.get_by_key")),
            ComponentResponse::ListenToChannel(Channel::from_string("data.set")),
        ]
    }
    fn message_received(&mut self, _poll: &Poll, channel: &Channel, message: &Value) -> Vec<ComponentResponse> {
        if channel == "data.get_by_key" {
            let key = match message.as_object().map(|o| o.get("key")) {
                Some(Some(&Value::String(ref str))) => str,
                _ => {
                    println!("Could not get key from {:?}", message);
                    return Vec::new();
                }
            };
            // println!("Getting data with key {:?}", key);
            let result: String = self.connection.get(key).unwrap_or_else(|_|String::from("{}"));
            let mut message = Message::new_emit(format!("data.{}", key), |_|{});
            
            message.data = if let Ok(value) = serde_json::from_str(&result) {
                value
            } else {
                Value::String(result)
            };
            vec![ComponentResponse::Send(message)]
        } else {
            let key = match message.as_object().map(|o| o.get("key")) {
                Some(Some(&Value::String(ref str))) => str,
                _ => {
                    println!("Could not get key from {:?}", message);
                    return Vec::new();
                }
            };
            let value = match message.as_object().map(|o| o.get("value")) {
                Some(Some(value)) => value.clone(),
                _ => {
                    println!("Could not get value from {:?}", message);
                    return Vec::new();
                }
            };
            self.connection.set::<&String, String, ()>(key, value.to_string()).unwrap();

            let mut message = Message::new_emit(format!("data.{}", key), |_|{});
            message.data = value;

            vec![ComponentResponse::Send(message)]
        }
    }
}
