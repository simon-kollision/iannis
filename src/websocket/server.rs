use simple_websockets::{Event, Responder};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::sync::mpsc::{Sender, Receiver};
use std::thread;

extern crate serde;
use serde::{Deserialize, Serialize};
use serde_json;

use crate::websocket::message::*;

pub struct Server {}

impl Server {
	pub fn new() -> Server {
		Server { }
	}

	pub fn run(&self, port: u16, channel_out: Sender<ClientMessage>, channel_in: Receiver<ServerMessage>){

		let clients = Arc::new(Mutex::new(HashMap::new()));
		let clients_clone = clients.clone();

		thread::spawn(move || {
			let eventhub = simple_websockets::launch(port).expect("Failed to open websocket port!");

			loop {
				match eventhub.poll_event(){
					Event::Connect(client_id, responder) => {
						println!("A client connected with id #{}", client_id);

						clients.lock().unwrap().insert(client_id, responder);
					},
					Event::Disconnect(client_id) => {
						println!("Client #{} disconnected.", client_id);

						clients.lock().unwrap().remove(&client_id);
					},
					Event::Message(client_id, message) => {
						if let simple_websockets::Message::Text(string) = message {

							let maybe_parsed = serde_json::from_str(&string);

							match maybe_parsed {
								Ok(client_message) => {
									channel_out.send(client_message).unwrap();
								},
								Err(e) => {
									println!("Error when attempting to parse incoming websocket message");
								}
							}

						}
					},
				}
			}
		});

		thread::spawn(move || {
			loop {
				let received = channel_in.recv();

				match received {
					Ok(server_message) => {
						let maybe_serialized = serde_json::to_string(&server_message);

						match maybe_serialized {
							Ok(serialized_string) => {
								let mut unlocked_clients = clients_clone.lock().unwrap();

								for (client_id, responder) in unlocked_clients.iter_mut() {
									responder.send(simple_websockets::Message::Text(serialized_string.clone()));
								}
							},
							Err(e) => {
								println!("Error when serialize server message");
							}
						}
					},
					Err(e) => {
						println!("Error when trying to receive server message from channel");
					}
				}
			}
		});
	}
}