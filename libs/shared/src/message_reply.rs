use serde_json::{Map, Value};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum MessageReply {
    None,
    ID(Uuid),
    Reply(Uuid)
}

impl MessageReply {
    pub fn inject_into(&self, map: &mut Map<String, Value>) {
        match *self {
            MessageReply::None => {},
            MessageReply::ID(ref id) => {
                map.insert(String::from("id"), Value::String(id.to_string()));
            },
            MessageReply::Reply(ref id) => {
                map.insert(String::from("reply_to"), Value::String(id.to_string()));
            }
        }
    }

    pub fn from_value(value: &Value) -> MessageReply {
        if let Some(Some(&Value::String(ref id))) = value.as_object().map(|o| o.get("id")) {
            if let Ok(uuid) = Uuid::parse_str(id) {
                return MessageReply::ID(uuid);
            }
            println!("Could not parse {:?} as a valid UUID", id);
        }
        if let Some(Some(&Value::String(ref id))) = value.as_object().map(|o| o.get("reply_to")) {
            if let Ok(uuid) = Uuid::parse_str(id) {
                return MessageReply::Reply(uuid);
            }
            println!("Could not parse {:?} as a valid UUID", id);
        }
        MessageReply::None
    }

    pub fn is_reply(&self) -> bool {
        match self {
            &MessageReply::Reply(_) => true,
            _ => false
        }
    }
}