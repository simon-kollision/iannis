use std::ptr;
use std::fmt;
use std::collections::HashMap;
use std::collections::VecDeque;
use crate::core::heaped::Heaped;

pub const BUFFER_SIZE: usize = 256;

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub struct NodeId(usize);

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
struct NodeEdge {
	from: Heaped<Node>,
	from_out_idx: usize,
	to: Heaped<Node>,
	to_in_idx: usize,

	from_buffer: *const Vec<f32>
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

struct Node {
	name: String,
	id: NodeId,

	ins: Vec<NodeIn>,
	outs: Vec<NodeOut>,

	edges_in: Vec<NodeEdge>,
	edges_out: Vec<NodeEdge>,

	behavior: Box<dyn NodeBehavior>,
	behavior_info: NodeBehaviorInfo
}

impl Node {
	fn new(name: String, id: NodeId, behavior: Box<dyn NodeBehavior>) -> Node {
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
			edges_out: Vec::new(),

			behavior_info: info
		}
	}

	fn destroy(&mut self) -> &mut Box<dyn NodeBehavior>{
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

	fn get_output_buffer(&mut self, output_idx: usize) -> *mut Vec<f32> {
		if let Some(outp) = self.outs.get_mut(output_idx) {
			return &mut outp.buffer
		} else {
			panic!("Trying to get an output buffer that doesn't exist!")
		}
	}

	fn add_input_edge(&mut self, edge: NodeEdge){
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

	fn add_output_edge(&mut self, edge: NodeEdge){
		if edge.from.mut_ptr != self {
			panic!("Trying to add an output edge to the wrong node!")
		}

		self.edges_out.push(edge);
	}

	fn remove_input_edge(&mut self, edge: &NodeEdge){
		if let Some(pos) = self.edges_in.iter().position(|e| e == edge) {
			self.edges_in.swap_remove(pos);

			let inp = self.ins.get_mut(edge.to_in_idx).unwrap();
			inp.from_buffer = ptr::null();

		} else {
			panic!("Trying to remove an input edge that doesn't exist!")
		}
	}

	fn remove_output_edge(&mut self, edge: &NodeEdge){
		if let Some(pos) = self.edges_out.iter().position(|e| e == edge) {
			self.edges_out.swap_remove(pos);
		} else {
			panic!("Trying to remove an output edge that doesn't exist!")
		}
	}

	fn update_inputs(&mut self){
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

	fn update(&mut self){
		self.update_inputs();

		self.behavior.update(&self.ins, &mut self.outs);
	}
}

pub struct NodeGraph {
	nodes: Vec<Heaped<Node>>,
	map: HashMap<NodeId, Heaped<Node>>,
	sorted: Vec<Heaped<Node>>,

	prev_node_id_val: usize,
	is_dirty: bool
}

impl NodeGraph {
	pub fn new() -> NodeGraph {
		NodeGraph {
			nodes: Vec::new(),
			map: HashMap::new(),
			sorted: Vec::new(),

			prev_node_id_val: 0,
			is_dirty: false
		}
	}

	pub fn add_node(&mut self, name: &str, behavior: Box<dyn NodeBehavior>) -> NodeId {
		println!("Adding node '{}'", name);

		self.prev_node_id_val += 1;
		let id = NodeId(self.prev_node_id_val);

		let node = Heaped::new_with_value(Node::new(name.to_string(), id, behavior));

		self.nodes.push(node);
		self.map.insert(id, node);

		self.is_dirty = true;

		return id;
	}

	pub fn add_node_by_recipe(&mut self, name: &str, recipe_name: &str) -> NodeId {
		let behavior = get_node_by_recipe(recipe_name);

		self.add_node(name, behavior)
	}

	pub fn remove_node(&mut self, node_id: NodeId){
		if let Some(heaped_node) = self.map.remove(&node_id){
			unsafe {
				let mut behavior = (*heaped_node.mut_ptr).destroy();
				behavior.before_drop();
			}

			if let Some(pos) = self.nodes.iter().position(|heaped| *heaped == heaped_node) {
				self.nodes.swap_remove(pos);
			}

		} else {
			panic!("Tried to remove a node that doesn't exist!");
		}

		self.is_dirty = true;
	}

	pub fn connect(&mut self, from: NodeId, from_out_idx: usize, to: NodeId, to_in_idx: usize) {
		let from_heaped: Heaped<Node>;
		let to_heaped: Heaped<Node>;

		if let Some(heaped_from) = self.map.get(&from) {
			from_heaped = heaped_from.clone();
		} else {
			panic!("Trying to connect from a non-existing node!");
		}

		if let Some(heaped_to) = self.map.get(&to) {
			to_heaped = heaped_to.clone();
		} else {
			panic!("Trying to connect to a non-existing node!");
		}

		let from_buffer: *mut Vec<f32>;

		unsafe {
			from_buffer = (*from_heaped.mut_ptr).get_output_buffer(from_out_idx);
		}

		let edge = NodeEdge {
			from: from_heaped,
			from_out_idx: from_out_idx,

			to: to_heaped,
			to_in_idx: to_in_idx,

			from_buffer: from_buffer
		};

		unsafe {
			(*from_heaped.mut_ptr).add_output_edge(edge);
			(*to_heaped.mut_ptr).add_input_edge(edge);
		}

		self.is_dirty = true;
	}

	pub fn disconnect(&mut self, from: NodeId, from_idx: usize, to: NodeId, to_idx: usize) {
		// TODO
	}

	fn sort(&mut self){
		let mut num_in_edges_map: HashMap<NodeId, usize> = HashMap::new();
		let mut queue: VecDeque<Heaped<Node>> = VecDeque::new();

		for heaped_node in self.nodes.iter_mut() {
			let node: &Node = unsafe { & *heaped_node.const_ptr };
			let num_in_edges = node.edges_in.len();

			num_in_edges_map.insert(node.id, num_in_edges);

			if num_in_edges == 0 {
				queue.push_back(heaped_node.clone());
			}
		}

		self.sorted.clear();
		while !queue.is_empty() {
			let front = queue.pop_front().unwrap();
			self.sorted.push(front.clone());

			let node: &Node = unsafe { & *front.const_ptr };
			for edge_out in node.edges_out.iter() {
				let to_node: &Node = unsafe { & *edge_out.to.const_ptr };

				let num_in_edges_left: &mut usize = num_in_edges_map.get_mut(&to_node.id).unwrap();
				*num_in_edges_left -= 1;

				if *num_in_edges_left == 0 {
					queue.push_back(edge_out.to.clone());
				}
			}
		}

		self.is_dirty = false;
	}

	pub fn update(&mut self) {
		if self.is_dirty {
			self.sort();
		}

		for heaped_node in self.sorted.iter_mut() {
			let node: &mut Node = unsafe {&mut *heaped_node.mut_ptr };
			node.update();
		}
	}

	pub fn to_dot(&mut self) -> String {
		if self.is_dirty {
			self.sort();
		}

		let mut result = String::from("digraph {\n");

		result += "\tnode [shape=box]\n\n";

		for (i, heaped_node) in self.sorted.iter().enumerate() {
			let node: &Node = unsafe {& *heaped_node.const_ptr };

			result = std::format!("{}\t{} [label=\"{}) {}\"]\n", result, node.id, i, node.name);
		}

		result += "\n";

		for heaped_node in self.sorted.iter() {
			let node: &Node = unsafe {& *heaped_node.const_ptr };

			for edge_out in node.edges_out.iter() {
				let to: &Node = unsafe {& *edge_out.to.const_ptr };

				result = std::format!("{}\t{} -> {}\n", result, node.id, to.id);
			}
		}

		result += "}";

		return result;
	}
}

type NodeRecipeFn = dyn FnMut() -> Box<dyn NodeBehavior>; 

static mut NODE_COOKBOOK: Option<HashMap<String, Box<NodeRecipeFn>>> = None;

pub fn register_node_recipe(name: &str, recipe: Box<NodeRecipeFn>){
	unsafe {
		if NODE_COOKBOOK.is_none() {
			NODE_COOKBOOK = Some(HashMap::new());
		}

		if let Some(cookbook) = &mut NODE_COOKBOOK {
			if cookbook.contains_key(&name.to_string()) {
				panic!("Trying to register a node recipe that already exists! Recipe was {}", name);
			} else {
				cookbook.insert(name.to_string(), recipe);
			}
		}
	}
}

pub fn get_node_by_recipe(name: &str) -> Box<dyn NodeBehavior> {
	unsafe {
		if let Some(cookbook) = &mut NODE_COOKBOOK {
			let mut maybe_recipe = cookbook.get_mut(&name.to_string());

			if let Some(recipe_fn) = &mut maybe_recipe {
				return recipe_fn()
			} else {
				panic!("Couldn't find a node recipe with the following name: {}", name);
			}
		} else {
			panic!("Node cookbook has not been initialized!")
		}
	}
}