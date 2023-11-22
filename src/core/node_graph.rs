use std::collections::HashMap;
use std::collections::VecDeque;
use crate::core::heaped::Heaped;
use crate::core::node::{Node, NodeBehavior, NodeEdge, NodeId};

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

    /*
	pub fn add_node_by_recipe(&mut self, name: &str, recipe_name: &str) -> NodeId {
		let behavior = get_node_by_recipe(recipe_name);

		self.add_node(name, behavior)
	}
    */

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