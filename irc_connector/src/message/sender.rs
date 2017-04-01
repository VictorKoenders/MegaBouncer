use shared::prelude::{Map, Value};

#[derive(Debug)]
pub struct Sender {
    pub name: String,
    pub host: String,
    pub flags: Vec<char>,
}

impl Sender {
    pub fn parse(string: &str) -> Sender {
        if let Some(p) = string.bytes().position(|b| b == b'!') {
            Sender {
                name: String::from(&string[..p]),
                host: String::from(&string[p + 1..]),
                flags: Vec::new()
            }
        } else {
            Sender {
                name: String::from(string),
                host: String::new(),
                flags: Vec::new()
            }
        }
    }
    pub fn to_json(&self) -> Value {
        Value::Object({
            let mut map = Map::new();
            map.insert(String::from("name"), Value::String(self.name.clone()));
            map.insert(String::from("host"), Value::String(self.host.clone()));
            map
        })
    }
}