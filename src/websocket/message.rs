extern crate serde;
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", content = "data")]
pub enum ClientMessage {
	AddNode(AddNodeMessage),
	RemoveNode(RemoveNodeMessage),

	ConnectNodes(ConnectNodesMessage),
	DisconnectNodes(DisconnectNodesMessage)
}

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type", content = "data")]
pub enum ServerMessage {
	Alright(AlrightMessage)
	//GraphStatus(GraphStatusMessage)
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AddNodeMessage {
	node_type: String
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RemoveNodeMessage {
	node_id: String
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ConnectNodesMessage {
	from_id: String,
	to_id: String,
	output_idx: usize,
	input_idx: usize
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DisconnectNodesMessage {
	from_id: String,
	to_id: String,
	output_idx: usize,
	input_idx: usize
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AlrightMessage {
	pub message: String
}