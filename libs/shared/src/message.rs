use super::{Channel, MessageReply};
use serde_json::{Map, Value, to_vec};
use std::error::Error as StdError;
use super::error::{Error, Result};
use std::convert::TryFrom;
//use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Message {
    pub id: MessageReply,
    pub sender: Option<String>,
    pub channel: Option<Channel>,
    pub data: Value,
}

impl TryFrom<Message> for Vec<u8> {
    type Error = super::error::Error;

    fn try_from(message: Message) -> Result<Vec<u8>> {
        to_vec(&message.to_json()).map_err(From::from)
    }
}

impl Message {
    pub fn to_json(&self) -> Value {
        let mut obj = Map::new();
        self.id.inject_into(&mut obj);
        if let Some(ref sender) = self.sender {
            obj.insert(String::from("sender"), Value::String(sender.clone()));
        }
        if let Some(ref channel) = self.channel {
            obj.insert("channel".to_owned(), Value::String(channel.to_string()));
        }
        obj.insert("data".to_owned(), self.data.clone());
        Value::Object(obj)
    }

    pub fn new_connected_client(name: String) -> Message {
        Message::new_emit("server.client.connected", |mut map| {
            map.insert(String::from("name"), Value::String(name));
        })
    }

    pub fn new_disconnected_client(name: String) -> Message {
        Message::new_emit("server.client.disconnected", |mut map| {
            map.insert(String::from("name"), Value::String(name));
        })
    }

    pub fn from_error<E: StdError>(error: E) -> Message {
        let mut obj = Map::new();
        obj.insert("message".to_owned(),
                   Value::String(error.description().to_string()));
        Message {
            id: MessageReply::None,
            sender: None,
            channel: Some(::channel::ERROR.clone()),
            data: Value::Object(obj),
        }
    }

    pub fn from_error_with_description<E: StdError, T: ToString>(error: E, str: T) -> Message {
        let mut obj = Map::new();
        obj.insert("message".to_owned(),
                   Value::String(error.description().to_string()));
        obj.insert("description".to_owned(), Value::String(str.to_string()));
        Message {
            id: MessageReply::None,
            sender: None,
            channel: Some(::channel::ERROR.clone()),
            data: Value::Object(obj),
        }
    }
    pub fn from_error_string<T: ::std::string::ToString>(error: T) -> Message {
        let mut obj = Map::new();
        obj.insert("message".to_owned(), Value::String(error.to_string()));
        Message {
            id: MessageReply::None,
            sender: None,
            channel: Some(::channel::ERROR.clone()),
            data: Value::Object(obj),
        }
    }

    pub fn from_json(value: Value) -> Result<Message> {
        let v = &value;
        let value = value.as_object()
            .ok_or_else(|| Error::new_invalid_json("JSON value should be an object"))?;

        let channel = value.get("channel")
            .and_then(|c| c.as_str())
            .map(Channel::from_string);

        let data: Value = value.get("data").map(|d| d.clone()).unwrap_or_else(|| Value::Null);

        let id = MessageReply::from_value(v);
        Ok(Message {
            id: id,
            sender: None,
            channel: channel,
            data: data,
        })
    }

    pub fn new_no_reply_target_found(original_message: Message) -> Message {
        Message {
            id: MessageReply::None,
            sender: None,
            channel: Some(::channel::ERROR.clone()),
            data: Value::Object({
                                    let mut map = Map::new();
                                    map.insert(String::from("message"),
                                               Value::String(String::from("Could not find reply")));
                                    map.insert(String::from("original"),
                                               original_message.to_json());
                                    map
                                }),
        }
    }

    pub fn new_identify<T: ToString>(name: &T) -> Message {
        Message {
            id: MessageReply::None,
            sender: None,
            channel: Some(::channel::IDENTIFY.clone()),
            data: Value::Object({
                                    let mut map = Map::new();
                                    map.insert(String::from("name"),
                                               Value::String(name.to_string()));
                                    map
                                }),
        }
    }

    pub fn new_register_listener<T: ToString>(channel: &T) -> Message {
        Message {
            id: MessageReply::None,
            sender: None,
            channel: Some(::channel::REGISTER_LISTENER.clone()),
            data: Value::Object({
                let mut map = Map::new();
                map.insert(String::from("channel"), Value::String(channel.to_string()));
                map
            }),
        }
    }

    pub fn new_reply<T: ToString>(channel: &T, uuid: Uuid, value: Value) -> Message {
        Message {
            id: MessageReply::Reply(uuid),
            sender: None,
            channel: Some(Channel::from_string(channel.to_string())),
            data: value,
        }
    }

    pub fn new_forget_listener<T: ToString>(channel: &T) -> Message {
        Message {
            id: MessageReply::None,
            sender: None,
            channel: Some(::channel::FORGET_LISTENER.clone()),
            data: Value::Object({
                let mut map = Map::new();
                map.insert(String::from("channel"), Value::String(channel.to_string()));
                map
            }),
        }
    }

    pub fn new_emit<T: ToString, C: FnOnce((&mut Map<String, Value>))>(channel: T,
                                                                       callback: C)
                                                                       -> Message {
        Message {
            id: MessageReply::None,
            sender: None,
            channel: Some(Channel::from_string(channel)),
            data: Value::Object({
                                    let mut map = Map::new();
                                    callback(&mut map);
                                    map
                                }),
        }
    }

    pub fn new_emit_with_id<T: ToString, C: FnOnce((&mut Map<String, Value>))>(channel: T,
    callback: C) -> Message {
        let mut message = Message::new_emit(channel, callback);
        let id = Uuid::new_v4();
        message.id = MessageReply::ID(id);
        message
    }
}