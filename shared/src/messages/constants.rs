/// The field on a message that indicates what type of message we're dealing with. This will be compared against the channels that the clients are listening to. If this action matches any channel a client is listening to, it will receive this message
pub const FIELD_ACTION: &str = "action";

/// The field that a message can have when it's send by a node. It contains the name of that node.
pub const FIELD_NODE_NAME: &str = "node_name";

/// The field that a message can have when it's send by a node. It contains the id of that node.
pub const FIELD_NODE_ID: &str = "node_id";

/// Indicates that a message is requesting a list of all connected nodes.
pub const ACTION_NODE_IDENTIFY: &str = "node.identify";

/// The response to a request of a list of all connected nodes.
pub const ACTION_RESPONSE_NODE_IDENTIFY: &str = "node.identified";

/// Indicates that a message is requesting a list of all connected nodes.
pub const ACTION_NODE_LIST: &str = "node.list";

/// The response to a request of a list of all connected nodes.
pub const ACTION_RESPONSE_NODE_LIST: &str = "node.listed";

/// Indicates that a message is registering to a channel. All subsequent messages that match this channel will be send to this client.
pub const ACTION_NODE_REGISTER_LISTENER: &str = "node.channel.register";

/// The response to a client registering a listener
pub const ACTION_RESPONSE_NODE_REGISTER_LISTENER: &str = "node.channel.registered";

/// An event triggered by the server when a node is disconnected
pub const EVENT_NODE_DISCONNECTED: &str = "node.disconnected";

