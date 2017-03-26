use serde_json::{Map, Value, to_vec};
use std::error::Error as StdError;
use super::error::{Error, Result};
use super::{ActionType, Channel};
use std::convert::TryFrom;

#[derive(Debug, Clone)]
pub struct Message {
    pub action: ActionType,
    pub channel: Option<Channel>,
    pub data: Value,
}

impl TryFrom<Message> for Vec<u8> {
    type Error = super::error::Error;

    fn try_from(message: Message) -> Result<Vec<u8>> {
        let mut obj = Map::new();
        obj.insert(String::from("action"), Value::String(message.action.to_string()));
        if let Some(ref channel) = message.channel {
            obj.insert("channel".to_owned(), Value::String(channel.to_string()));
        }
        obj.insert("data".to_owned(), message.data.clone());
        let json = Value::Object(obj);
        to_vec(&json).map_err(From::from)
    }
}

impl Message {
    // pub fn to_bytes(&self) -> Result<Vec<u8>> {
    //     let mut obj = Map::new();
    //     obj.insert(String::from("action"), Value::String(self.action.to_string()));
    //     if let Some(ref channel) = self.channel {
    //         obj.insert("channel".to_owned(), Value::String(channel.to_string()));
    //     }
    //     obj.insert("data".to_owned(), self.data.clone());
    //     let json = Value::Object(obj);
    //     to_vec(&json).map_err(From::from)
    // }

    pub fn from_error<E: StdError>(error: E) -> Message {
        let mut obj = Map::new();
        obj.insert("message".to_owned(), Value::String(error.description().to_string()));
        Message {
            action: ActionType::Error,
            channel: None,
            data: Value::Object(obj),
        }
    }

    pub fn from_error_with_description<E: StdError, T: ToString>(error: E, str: T) -> Message {
        let mut obj = Map::new();
        obj.insert("message".to_owned(), Value::String(error.description().to_string()));
        obj.insert("description".to_owned(), Value::String(str.to_string()));
        Message {
            action: ActionType::Error,
            channel: None,
            data: Value::Object(obj),
        }
    }
    pub fn from_error_string<T: ::std::string::ToString>(error: T) -> Message {
        let mut obj = Map::new();
        obj.insert("message".to_owned(), Value::String(error.to_string()));
        Message {
            action: ActionType::Error,
            channel: None,
            data: Value::Object(obj),
        }
    }

    pub fn from_json(value: Value) -> Result<Message> {
        let value = value.as_object()
            .ok_or_else(||Error::new_invalid_json("JSON value should be an object"))?;

        let action = value
            .get("action")
            .ok_or_else(||Error::new_invalid_json("Root JSON object needs an action field"))?
            .as_str()
            .ok_or_else(||Error::new_invalid_json("Action field needs to be a string"))?;
        
        let action = ActionType::from_str(action)
            .ok_or_else(|| Error::new_invalid_json(format!("Invalid action field, needs to be one of: {}", ActionType::default_types().join(", "))))?;

        let channel = value
            .get("channel")
            .and_then(|c| c.as_str())
            .unwrap_or_else(||"")
            .to_owned();

        let data = value
            .get("data")
            .ok_or_else(||Error::new_invalid_json("Root JSON object needs a data field"))?
            .clone();

        Ok(Message {
            action: action,
            channel: Some(Channel::from_string(channel)),
            data: data
        })
    }
    
    pub fn new_identify<T: ToString>(name: &T) -> Message {
        Message {
            action: ActionType::Identify,
            channel: None,
            data: Value::Object({
                let mut map = Map::new();
                map.insert(String::from("name"), Value::String(name.to_string()));
                map
            }),
        }
    }
    
    pub fn new_register_listener<T: ToString>(channel: &T) -> Message {
        Message {
            action: ActionType::RegisterListener,
            channel: Some(Channel::from_string(channel.to_string())),
            data: Value::Null,
        }
    }
    
    pub fn new_forget_listener<T: ToString>(channel: &T) -> Message {
        Message {
            action: ActionType::ForgetListener,
            channel: Some(Channel::from_string(channel.to_string())),
            data: Value::Null,
        }
    }

    pub fn new_emit<T: ToString, C: FnOnce((&mut Map<String, Value>))>(channel: T, callback: C) -> Message {
        Message {
            action: ActionType::Emit,
            channel: Some(Channel::from_string(channel)),
            data: Value::Object({
                let mut map = Map::new();
                callback(&mut map);
                map
            })
        }
    }
}