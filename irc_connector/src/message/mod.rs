mod sender;
mod target;
pub use self::sender::Sender;
pub use self::target::Target;

#[derive(Debug)]
pub enum Message {
    Ping(String),
    Pong(String),
    Notice(Sender, Target, String),
    Privmsg(Sender, Target, String),
    Numeric(Sender, u16, String),
    Mode(Sender, Target, Vec<char>),
}

impl Message {
    pub fn from_line(line: &str) -> Option<Message> {
        let mut iter = line.split_whitespace();
        let from = match iter.next() {
            Some(from) => from,
            None => {
                println!("Could not retreive sender: {:?}", line);
                return None;
            }
        };

        if from == "PING" {
            let remaining = iter.map(|str| format!("{} ", str)).collect::<String>();
            return Some(Message::Ping(remaining));
        }

        if from == "PONG" {
            let remaining = iter.map(|str| format!("{} ", str)).collect::<String>();
            return Some(Message::Pong(remaining));
        }
        let from = Sender::parse(from);

        let line_type = match iter.next() {
            Some(line_type) => line_type,
            None => {
                println!("Could not retreive line type: {:?}", line);
                return None;
            }
        };
        let target = match iter.next() {
            Some(target) => Target::parse(target),
            None => {
                println!("Could not retreive PRIVMSG target: {:?}", line);
                return None;
            }
        };
        let remaining = iter.map(|str| format!("{} ", str)).collect::<String>();
        let remaining = if let Some(b':') = remaining.bytes().next() { String::from(&remaining[1..]) } else { remaining };

        if line_type == "PRIVMSG" {
            Some(Message::Privmsg(from, target, remaining))
        } else if let Ok(nr) = line_type.parse::<u16>() {
            Some(Message::Numeric(from, nr, remaining))
        } else if line_type == "NOTICE" {
            Some(Message::Notice(from, target, remaining))
        } else if line_type == "MODE" {
            Some(Message::Mode(from, target, remaining.trim().chars().skip(1).collect()))
        } else {
            println!("Unknown line type {:?}", line_type);
            None
        }
    }
}

impl ToString for Message {
    fn to_string(&self) -> String {
        match self {
            &Message::Ping(ref str) => format!("PING {}", str),
            &Message::Pong(ref str) => format!("PONG {}", str),
            &Message::Notice(_, ref target, ref message) => format!("NOTICE {} :{}", target.to_string(), message),
            &Message::Privmsg(_, ref target, ref message) => format!("PRIVMSG {} :{}", target.to_string(), message),
            &Message::Mode(_, ref target, ref modes) => format!("MODE {} +{}", target.to_string(), modes.iter().collect::<String>()),
            &Message::Numeric(_, ref _number, ref _message) => unreachable!()
        }
    }
}
