use shared::Value;
pub struct IrcChannel {
    pub name: String
}

impl IrcChannel {
    pub fn from_json(value: &Value) -> Option<IrcChannel> {
        let name = match value.get("name").and_then(Value::as_str) { Some(s) => s.to_string(), None => return None };

        Some(IrcChannel {
            name: name
        })
    }
}