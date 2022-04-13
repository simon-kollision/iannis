use std::fs::File;
use std::io::Write;
use std::process::Command;
use std::thread;

mod core;
mod behavior;
use crate::core::node::*;
use crate::core::audio::*;
use crate::behavior::waveform::*;
use crate::behavior::basic::*;
use crate::core::heaped::Heaped;
use crate::core::websocket::*;

extern crate ringbuf;

fn write_file(path: &str, data: &String){
    let mut output = File::create(path).unwrap();
    write!(output, "{}", data);
}

fn run_dot(dot_path: &str, png_path: &str){
    Command::new("/opt/homebrew/bin/dot").arg("-Tpng").arg(dot_path).arg("-o").arg(png_path).status();
}

fn main() {
    let ringbuf_buffer_size = (BUFFER_SIZE*32).try_into().expect("Ringbuf BUFFER_SIZE cannot fit into usize!");

    let mut ringbuf = ringbuf::RingBuffer::<f32>::new(ringbuf_buffer_size);
    let (mut ringbuf_prod, mut ringbuf_cons) = ringbuf.split();

    let mut audio_manager = AudioManager::new();

    let graph_thread = thread::spawn(move || {
        let mut graph = NodeGraph::new();

        let sin = graph.add_node(String::from("sin"), Box::new(SinNode::new(440.0)));
        let sin_b = graph.add_node(String::from("sin"), Box::new(SinNode::new(100.0)));
        let sin_c = graph.add_node(String::from("sin"), Box::new(SinNode::new(630.0)));
        let product = graph.add_node(String::from("product"), Box::new(ProductNode::new(3)));

        let output_buffer: Heaped<Vec<f32>> = Heaped::new_with_value(vec![0.0; BUFFER_SIZE*2]);
        let output = graph.add_node(String::from("output"), Box::new(InterleavingOutputNode::new(output_buffer)));

        graph.connect(sin, 0, product, 0);
        graph.connect(sin_b, 0, product, 1);
        graph.connect(sin_c, 0, product, 2);
        graph.connect(product, 0, output, 0);
        graph.connect(product, 0, output, 1);

        write_file("graph.dot", &graph.to_dot());
        run_dot("graph.dot", "graph.png");

        loop {
            let remaining = ringbuf_prod.remaining();
            if remaining > BUFFER_SIZE {

                graph.update();

                unsafe {
                    ringbuf_prod.push_slice(&(*output_buffer.const_ptr));
                }
            } else {
                thread::park();
            }
        }
    });

    audio_manager.open_output_stream(ringbuf_cons, graph_thread.thread().clone());

    let mut websocket_server = WebsocketServer::new("127.0.0.1:9001".to_string());
    websocket_server.run();
}
