use crate::core::node::{NodeBehavior, NodeBehaviorInfo, NodeIn, NodeOut};

pub struct WaveformNode {
    waveform: Vec<f32>
}

impl WaveformNode {
    pub fn new(waveform: Vec<f32>) -> WaveformNode {
        WaveformNode {
            waveform: waveform
        }
    }
}

impl NodeBehavior for WaveformNode {
    fn get_info(&self) -> NodeBehaviorInfo {
        NodeBehaviorInfo {
            type_name: String::from("WaveformNode"),
            num_ins: 0,
            num_outs: 1
        }
    }

    fn update(&mut self, inputs: &Vec<NodeIn>, outputs: &mut Vec<NodeOut>){
        let output = outputs.get_mut(0).unwrap();
        let k = self.waveform.len();

        for n in 0..output.buffer.len() {
            output.buffer[n] = self.waveform[n%k];
        }
    }

    fn before_drop(&mut self){

    }
}

pub struct SinNode {
    freq: f32,
    clock: f32
}

impl SinNode {
    pub fn new(freq: f32) -> SinNode {
        SinNode {
            freq: freq,
            clock: 0.0,
        }
    }
}

impl NodeBehavior for SinNode {
    fn get_info(&self) -> NodeBehaviorInfo {
        NodeBehaviorInfo {
            type_name: String::from("SinNode"),
            num_ins: 0,
            num_outs: 1
        }
    }

    fn update(&mut self, inputs: &Vec<NodeIn>, outputs: &mut Vec<NodeOut>){
        let output = outputs.get_mut(0).unwrap();

        for n in 0..output.buffer.len() {
            output.buffer[n] = (self.clock * self.freq * 2.0 * std::f32::consts::PI / 44100.0).sin();
            self.clock = (self.clock + 1.0) % 44100.0;
        }
    }

    fn before_drop(&mut self){

    }
}