use crate::core::node::{NodeBehavior, NodeBehaviorInfo, NodeIn, NodeOut, BUFFER_SIZE};
use crate::core::heaped::Heaped;
extern crate ringbuf;

pub struct SumNode {
    num_ins: usize
}

impl SumNode {
    pub fn new(num_ins: usize) -> SumNode {
        SumNode {
            num_ins: num_ins
        }
    }
}

impl NodeBehavior for SumNode {
    fn get_info(&self) -> NodeBehaviorInfo {
        NodeBehaviorInfo {
            type_name: String::from("SumNode"),
            num_ins: self.num_ins,
            num_outs: 1
        }
    }

    fn update(&mut self, inputs: &Vec<NodeIn>, outputs: &mut Vec<NodeOut>){
        let output = outputs.get_mut(0).unwrap();
        let mut sum: f32;

        for n in 0..output.buffer.len() {
            sum = 0.0;

            for i in 0..inputs.len() {
                sum += inputs[i].buffer[n];
            }

            output.buffer[n] = sum;
        }
    }
}

pub struct ProductNode {
    num_ins: usize
}

impl ProductNode {
    pub fn new(num_ins: usize) -> ProductNode {
        ProductNode {
            num_ins: num_ins
        }
    }
}

impl NodeBehavior for ProductNode {
    fn get_info(&self) -> NodeBehaviorInfo {
        NodeBehaviorInfo {
            type_name: String::from("ProductNode"),
            num_ins: self.num_ins,
            num_outs: 1
        }
    }

    fn update(&mut self, inputs: &Vec<NodeIn>, outputs: &mut Vec<NodeOut>){
        let output = outputs.get_mut(0).unwrap();
        let mut sum: f32;

        for n in 0..output.buffer.len() {
            sum = 1.0;

            for i in 0..inputs.len() {
                sum *= inputs[i].buffer[n];
            }

            output.buffer[n] = sum;
        }
    }
}

pub struct InterleavingOutputNode {
    heaped_out_buffer: Heaped<Vec<f32>>
}

impl InterleavingOutputNode {
    pub fn new(mut out_buffer: Heaped<Vec<f32>>) -> InterleavingOutputNode {
        InterleavingOutputNode {
            heaped_out_buffer: out_buffer
        }
    }
}

impl NodeBehavior for InterleavingOutputNode {
    fn get_info(&self) -> NodeBehaviorInfo {
        NodeBehaviorInfo {
            type_name: String::from("InterleavingOutputNode"),
            num_ins: 2,
            num_outs: 0
        }
    }

    fn update(&mut self, inputs: &Vec<NodeIn>, outputs: &mut Vec<NodeOut>){
        let left = inputs.get(0).unwrap();
        let right = inputs.get(0).unwrap();

        unsafe {
        for i in 0..BUFFER_SIZE {
            (*self.heaped_out_buffer.mut_ptr)[i*2] = left.buffer[i];
            (*self.heaped_out_buffer.mut_ptr)[i*2 + 1] = right.buffer[i];
        }
        }

        //self.ringbuf_producer.push_slice(&self.tmp_buffer[0..]);
    }
}