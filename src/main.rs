use std::fs::File;
use std::io::Write;
use std::process::Command;
use std::thread;
use std::sync::mpsc::{Sender, Receiver};

mod core;
mod behavior;
mod websocket;
use crate::core::node::*;
use crate::core::audio::*;
use crate::core::heaped::Heaped;
use crate::behavior::waveform::*;
use crate::behavior::basic::*;
use crate::websocket::message::*;
use crate::websocket::server::Server;

use std::sync::mpsc::channel;

extern crate ringbuf;

fn write_file(path: &str, data: &String){
	let mut output = File::create(path).unwrap();
	write!(output, "{}", data);
}

fn run_dot(dot_path: &str, png_path: &str){
	Command::new("/opt/homebrew/bin/dot").arg("-Tpng").arg(dot_path).arg("-o").arg(png_path).status();
}

fn main() {
	let ringbuf_buffer_size = (BUFFER_SIZE*32).try_into().expect("Ringbuf BUFFER_SIZE*32 cannot fit into usize!");

	let mut ringbuf = ringbuf::RingBuffer::<f32>::new(ringbuf_buffer_size);
	let (mut ringbuf_prod, mut ringbuf_cons) = ringbuf.split();

	let mut audio_manager = AudioManager::new();

	let (ws_out_tx, ws_out_rx): (Sender<ClientMessage>, Receiver<ClientMessage>) = channel();
	let (ws_in_tx, ws_in_rx): (Sender<ServerMessage>, Receiver<ServerMessage>) = channel();

	let graph_thread = thread::spawn(move || {
		let mut graph = NodeGraph::new();

		let sin_freq = graph.add_node("sin_freq", Box::new(WaveformNode::new(vec![120.0])));
		let sin = graph.add_node("sin", Box::new(SinNode::new()));

		let sin_scale_amt = graph.add_node("sin_scale_amt", Box::new(WaveformNode::new(vec![0.1])));
		let sin_scale = graph.add_node("sin_scale", Box::new(ProductNode::new(2)));

		let sin_offset_amt = graph.add_node("sin_offset_amt", Box::new(WaveformNode::new(vec![1.0])));
		let sin_offset = graph.add_node("sin_offset", Box::new(SumNode::new(2)));

		let sin_2_freq = graph.add_node("sin_2_freq", Box::new(WaveformNode::new(vec![400.0])));
		let sin_2_freq_mod = graph.add_node("sin_2_freq_mod", Box::new(ProductNode::new(2)));

		let sin_2 = graph.add_node("sin2", Box::new(SinNode::new()));

		let left_amp_mod_freq = graph.add_node("left_amp_mod_freq", Box::new(WaveformNode::new(vec![0.87])));
		let left_amp_mod = graph.add_node("left_amp_mod", Box::new(SinNode::new()));

		let right_amp_mod_freq = graph.add_node("right_amp_mod_freq", Box::new(WaveformNode::new(vec![1.73])));
		let right_amp_mod = graph.add_node("right_amp_mod", Box::new(SinNode::new()));

		let left_amp = graph.add_node("left_amp", Box::new(ProductNode::new(2)));
		let right_amp = graph.add_node("right_amp", Box::new(ProductNode::new(2)));

		let output_buffer: Heaped<Vec<f32>> = Heaped::new_with_value(vec![0.0; BUFFER_SIZE*2]);
		let output = graph.add_node("output", Box::new(InterleavingOutputNode::new(output_buffer)));

		graph.connect(sin_freq, 0, sin, 0);
		graph.connect(sin, 0, sin_scale, 0);
		graph.connect(sin_scale_amt, 0, sin_scale, 1);

		graph.connect(sin_scale, 0, sin_offset, 0);
		graph.connect(sin_offset_amt, 0, sin_offset, 1);

		graph.connect(sin_offset, 0, sin_2_freq_mod, 0);
		graph.connect(sin_2_freq, 0, sin_2_freq_mod, 1);
		graph.connect(sin_2_freq_mod, 0, sin_2, 0);

		graph.connect(sin_2, 0, left_amp, 0);
		graph.connect(sin_2, 0, right_amp, 0);

		graph.connect(left_amp_mod_freq, 0, left_amp_mod, 0);
		graph.connect(left_amp_mod, 0, left_amp, 1);

		graph.connect(right_amp_mod_freq, 0, right_amp_mod, 0);
		graph.connect(right_amp_mod, 0, right_amp, 1);

		graph.connect(left_amp, 0, output, 0);
		graph.connect(right_amp, 0, output, 1);

		write_file("graph.dot", &graph.to_dot());
		run_dot("graph.dot", "graph.png");

		loop {
			let remaining = ringbuf_prod.remaining();
			if remaining > BUFFER_SIZE {
				graph.update();

				unsafe {
					ringbuf_prod.push_slice(&(*output_buffer.const_ptr));
				}
				println!(".");
			} else {
				println!("parking");
				thread::park();
			}
		}
	});

	audio_manager.open_output_stream(ringbuf_cons, graph_thread.thread().clone());

	let mut websocket_server: Server = Server::new();
	websocket_server.run(9001, ws_out_tx, ws_in_rx);

	loop {
		let maybe_ws_msg = ws_out_rx.recv();

		if let Ok(message_from_ws) = maybe_ws_msg {
			match message_from_ws {
				ClientMessage::AddNode(add_node_message) => {
					println!("Got add node message from client!");
					ws_in_tx.send(ServerMessage::Alright(AlrightMessage { message: "add node ok!".to_string() }));
				},
				ClientMessage::RemoveNode(remove_node_message) => {
					println!("Got remove node message from client!");
					ws_in_tx.send(ServerMessage::Alright(AlrightMessage { message: "remove node ok!".to_string() }));
				},
				ClientMessage::ConnectNodes(connect_nodes_message) => {
					println!("Got connect nodes message");
					ws_in_tx.send(ServerMessage::Alright(AlrightMessage { message: "connect nodes ok!".to_string() }));
				},
				ClientMessage::DisconnectNodes(disconnect_nodes_message) => {
					println!("Got disconnect nodes message!");
					ws_in_tx.send(ServerMessage::Alright(AlrightMessage { message: "disconnect nodes ok!".to_string() }));
				}
			}
		}
	}
}
