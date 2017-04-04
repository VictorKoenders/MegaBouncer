extern crate redis;
extern crate shared;

use shared::prelude::*;
use redis::{Client, Commands, Connection};

fn main() {
    // let client = Client::open("redis://redis-16682.c9.us-east-1-2.ec2.cloud.redislabs.com:16682").unwrap();
    // let con: Connection = client.get_connection().unwrap();
    // let count: i32 = con.get("counter").unwrap_or(0);
    // println!("Counter: {}", count);
    // con.set::<&str, i32, ()>("counter", count + 1).unwrap();
    let mut client = shared::Client::default();
    client.register::<RedisConnector>();
    client.start("datastore");
}

struct RedisConnector {
    connection: Connection
}

impl Default for RedisConnector {
    fn default() -> RedisConnector {
        let client = Client::open("redis://redis-16682.c9.us-east-1-2.ec2.cloud.redislabs.com:16682").unwrap();
        let con: Connection = client.get_connection().unwrap();
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
            let result: String = self.connection.get(key).unwrap_or_else(|_|String::new());
            println!("Key {:?} is {:?}", key, result);
            vec![ComponentResponse::Reply({
                let mut map = Map::new();
                map.insert(String::from("key"), Value::String(key.clone()));
                map.insert(String::from("value"), Value::String(result));
                Value::Object(map)
            })]
        } else {
            println!("{:?} {:?}", channel, message);
            Vec::new()
        }
    }
}
