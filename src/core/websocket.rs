use std::net::TcpListener;
use std::thread;

use tungstenite::accept;

enum ClientMessage {
	AddNode(AddNodeMessage),
	RemoveNode(RemoveNodeMessage),
	Connect(ConnectMessage),
	Disconnect(DisconnectMessage)
}

struct AddNodeMessage {
	node_type: String
}

struct RemoveNodeMessage {
	node_id: String
}

pub struct WebsocketServer {
	socket: TcpListener
}

impl WebsocketServer {
	pub fn new(addr: String) -> WebsocketServer {
		let socket = TcpListener::bind(addr).unwrap();

		WebsocketServer {
			socket: socket
		}
	}

	pub fn run(&mut self){
		for stream in self.socket.incoming() {		
			thread::spawn (move || {
				let mut websocket = accept(stream.unwrap()).unwrap();
				loop {
					let msg = websocket.read_message().unwrap();

					if msg.is_binary() || msg.is_text() {
						websocket.write_message(msg).unwrap();
					}
				}
			});
		}
	}
}