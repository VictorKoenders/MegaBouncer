
#[derive(Debug)]
pub enum Target {
    Channel(String),
    Person(String),
}

impl ToString for Target {
    fn to_string(&self) -> String {
        match *self {
            Target::Channel(ref channel) => format!("#{}", channel),
            Target::Person(ref person) => person.clone()
        }
    }
}

impl Target {
    pub fn parse(string: &str) -> Target {
        if let Some(b'#') = string.bytes().next() {
            Target::Channel(String::from(&string[1..]))
        } else {
            Target::Person(String::from(string))
        }
    }
}