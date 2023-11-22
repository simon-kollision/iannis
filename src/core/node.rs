use std::ptr;
use std::fmt;
use crate::core::heaped::Heaped;

pub const BUFFER_SIZE: usize = 256;

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub struct NodeId(pub(crate) usize);

impl fmt::Display for NodeId {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "node{}", self.0)
	}
}

pub struct NodeIn {
	pub buffer: Vec<f32>,
	from_buffer: *const Vec<f32>,
}

impl NodeIn {
	fn new() -> NodeIn {
		NodeIn { 
			buffer: vec![0.0; BUFFER_SIZE],
			from_buffer: ptr::null()
		}
	}
}

pub struct NodeOut {
	pub buffer: Vec<f32>
}

impl NodeOut {
	fn new() -> NodeOut {
		NodeOut { 
			buffer: vec![0.0; BUFFER_SIZE],
		}
	}
}

#[derive(Copy, Clone, PartialEq)]
pub(crate) struct NodeEdge {
	pub(crate) from: Heaped<Node>,
	pub(crate) from_out_idx: usize,
	pub(crate) to: Heaped<Node>,
	pub(crate) to_in_idx: usize,

	pub(crate) from_buffer: *const Vec<f32>
}

pub struct NodeBehaviorInfo {
	pub type_name: String,
	pub num_ins: usize,
	pub num_outs: usize
}

pub trait NodeBehavior {
	fn get_info(&self) -> NodeBehaviorInfo;
	fn update(&mut self, inputs: &Vec<NodeIn>, outputs: &mut Vec<NodeOut>);
	fn before_drop(&mut self){
		//
	}
}

pub(crate) struct Node {
	pub(crate) name: String,
	pub(crate) id: NodeId,

	pub(crate) ins: Vec<NodeIn>,
	pub(crate) outs: Vec<NodeOut>,

	pub(crate) edges_in: Vec<NodeEdge>,
	pub(crate) edges_out: Vec<NodeEdge>,

	pub(crate) behavior: Box<dyn NodeBehavior>
}

impl Node {
	pub(crate) fn new(name: String, id: NodeId, behavior: Box<dyn NodeBehavior>) -> Node {
		let info = behavior.get_info();

		let mut ins = Vec::with_capacity(info.num_ins);

		for _ in 0..info.num_ins {
			ins.push(NodeIn::new());
		}

		let mut outs = Vec::with_capacity(info.num_outs);

		for _ in 0..info.num_outs {
			outs.push(NodeOut::new());
		}

		Node { 
			name, 
			id,

			ins, 
			outs,
			
			behavior,

			edges_in: Vec::new(),
			edges_out: Vec::new()
		}
	}

	pub(crate) fn destroy(&mut self) -> &mut Box<dyn NodeBehavior>{
		for edge_in in self.edges_in.iter_mut() {
			unsafe {
				(*edge_in.from.mut_ptr).remove_output_edge(edge_in);
			}
		}

		for edge_out in self.edges_out.iter_mut() {
			unsafe {
				(*edge_out.to.mut_ptr).remove_input_edge(edge_out);
			}
		}

		return &mut self.behavior;
	}

	pub(crate) fn get_output_buffer(&mut self, output_idx: usize) -> *mut Vec<f32> {
		if let Some(outp) = self.outs.get_mut(output_idx) {
			return &mut outp.buffer
		} else {
			panic!("Trying to get an output buffer that doesn't exist!")
		}
	}

	pub(crate) fn add_input_edge(&mut self, edge: NodeEdge){
		if edge.to.mut_ptr != self {
			panic!("Trying to add an input edge to the wrong node!")
		}

		if let Some(inp) = self.ins.get_mut(edge.to_in_idx) {
			if !inp.from_buffer.is_null() {
				panic!("Trying to add an input edge but the input is already connected to something else!");
			}

			inp.from_buffer = edge.from_buffer;
			self.edges_in.push(edge);
		} else {
			panic!("Trying to add an input edge but the input does not exist!");
		}
	}

	pub(crate) fn add_output_edge(&mut self, edge: NodeEdge){
		if edge.from.mut_ptr != self {
			panic!("Trying to add an output edge to the wrong node!")
		}

		self.edges_out.push(edge);
	}

	pub(crate) fn remove_input_edge(&mut self, edge: &NodeEdge){
		if let Some(pos) = self.edges_in.iter().position(|e| e == edge) {
			self.edges_in.swap_remove(pos);

			let inp = self.ins.get_mut(edge.to_in_idx).unwrap();
			inp.from_buffer = ptr::null();

		} else {
			panic!("Trying to remove an input edge that doesn't exist!")
		}
	}

	pub(crate) fn remove_output_edge(&mut self, edge: &NodeEdge){
		if let Some(pos) = self.edges_out.iter().position(|e| e == edge) {
			self.edges_out.swap_remove(pos);
		} else {
			panic!("Trying to remove an output edge that doesn't exist!")
		}
	}

	pub(crate) fn update_inputs(&mut self){
		for inp in self.ins.iter_mut(){
			if inp.from_buffer.is_null() {
				inp.buffer.fill(0.0);

			} else {
				unsafe {
					inp.buffer.copy_from_slice(&(*inp.from_buffer)[0..]);
				}
			}
		}
	}

	pub(crate) fn update(&mut self){
		self.update_inputs();

		self.behavior.update(&self.ins, &mut self.outs);
	}
}