use client::Client;
use serde_json::{Map, Value};
use uuid::Uuid;

pub use shared::messages::constants::*;

/// Add the clients name and ID to the message
pub fn add_client_sender_to_message(message: &mut Value, client_name: &str, client_id: &Uuid) {
    if let Some(obj) = message.as_object_mut() {
        obj.insert(
            String::from(FIELD_NODE_NAME),
            Value::String(client_name.to_owned()),
        );
        obj.insert(
            String::from(FIELD_NODE_ID),
            Value::String(client_id.to_string()),
        );
    }
}

/// Create a ACTION_RESPONSE_NODE_LIST object, for when a client requests ACTION_NODE_LIST
pub fn make_node_list<'a>(nodes: impl Iterator<Item = &'a Client>) -> Value {
    Value::Object({
        let mut map = Map::new();
        map.insert(
            String::from(FIELD_ACTION),
            Value::String(String::from(ACTION_RESPONSE_NODE_LIST)),
        );
        map.insert(
            String::from("nodes"),
            Value::Array(
                nodes
                    .filter_map(|c| {
                        let mut map = Map::new();
                        map.insert(String::from("id"), Value::String(c.id.to_string()));
                        map.insert(
                            String::from("name"),
                            Value::String(c.name.as_ref()?.to_owned()),
                        );
                        map.insert(
                            String::from("channels"),
                            Value::Array(
                                c.listening_to.iter().cloned().map(Value::String).collect(),
                            ),
                        );
                        Some(Value::Object(map))
                    }).collect(),
            ),
        );
        map
    })
}

/// Create an error object with the given name
pub fn make_error(msg: &str) -> Value {
    Value::Object({
        let mut map = Map::new();
        map.insert(String::from(FIELD_ACTION), Value::String(String::from("error")));
        map.insert(String::from("message"), Value::String(String::from(msg)));
        map
    })
}

/// Create an "node.identified" object with the given name and id
pub fn make_client_joined(name: &str, uuid: &Uuid) -> Value {
    Value::Object({
        let mut map = Map::new();
        map.insert(
            String::from(FIELD_ACTION),
            Value::String(String::from(ACTION_RESPONSE_NODE_IDENTIFY)),
        );
        map.insert(String::from("name"), Value::String(String::from(name)));
        map.insert(String::from("id"), Value::String(uuid.to_string()));
        map
    })
}

/// Create an "node.disconnected" object with the given name and id
pub fn make_client_disconnected(name: &str, uuid: &Uuid) -> Value {
    Value::Object({
        let mut map = Map::new();
        map.insert(
            String::from(FIELD_ACTION),
            Value::String(String::from(EVENT_NODE_DISCONNECTED)),
        );
        map.insert(String::from("name"), Value::String(String::from(name)));
        map.insert(String::from("id"), Value::String(uuid.to_string()));
        map
    })
}

/// Create an "node.channel.registered" object with the given name, channel and id
pub fn make_client_listening_to(name: &str, channel: &str, uuid: &Uuid) -> Value {
    Value::Object({
        let mut map = Map::new();
        map.insert(
            String::from(FIELD_ACTION),
            Value::String(String::from(ACTION_RESPONSE_NODE_REGISTER_LISTENER)),
        );
        map.insert(String::from("name"), Value::String(String::from(name)));
        map.insert(
            String::from("channel"),
            Value::String(String::from(channel)),
        );
        map.insert(String::from("id"), Value::String(uuid.to_string()));
        map
    })
}
