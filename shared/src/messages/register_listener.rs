use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

pub struct RegisterListener(pub String);

impl Serialize for RegisterListener {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("RegisterListener", 2)?;
        s.serialize_field("action", "node.listener.register")?;
        s.serialize_field("channel", &self.0)?;
        s.end()
    }
}
