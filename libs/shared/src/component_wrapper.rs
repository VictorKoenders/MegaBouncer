use super::Component;
use channel::Channel;
use mio::Token;

pub struct ComponentWrapper {
    pub component: Box<Component>,
    pub channels: Vec<Channel>,
    pub tokens: Vec<Token>,
    pub removed: bool,
}

impl ComponentWrapper {
    pub fn from_component(component: Box<Component>) -> ComponentWrapper {
        ComponentWrapper {
            component: component,
            channels: Vec::new(),
            tokens: Vec::new(),
            removed: false
        }
    }
}