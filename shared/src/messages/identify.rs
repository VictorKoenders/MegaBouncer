use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

pub struct Identify(pub String);

impl Serialize for Identify {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("Identify", 2)?;
        s.serialize_field("action", "node.identify")?;
        s.serialize_field("name", &self.0)?;
        s.end()
    }
}
