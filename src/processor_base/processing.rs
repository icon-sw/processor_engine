use std::iter::Map;

use spmc::{Receiver,Sender};

use crate::processor_base::parameter::Parameter;
use crate::interfaces::InterfaceTrait;

pub trait DataTypeTrait: std::fmt::Debug + std::fmt::Display + Ord + Clone + Copy + Send + Sync {
    fn get_id(&self) -> i64;
    fn get_data(&self) -> Any;
}
pub struct ReceiverMultiplexer
{
    pub input_interfaces: Vec<Receiver<dyn DataTypeTrait>>,
    pub output_interface: Sender<dyn DataTypeTrait>,
    pub data_vector: Map<i64, Vec<Box<dyn DataTypeTrait>>>,
}

impl InputMultiplexer {
    pub fn new() -> Self {
        Self {
            input_interfaces: Vec::new(),
            output_interface: Sender::new(),
        }
    }
    pub fn append_input_interface(&mut self, new_interface: Receiver<dyn DataTypeTrait>) {
        self.input_interfaces.push(new_interface);
    }
    pub fn set_output_interface(&mut self, new_interface: Sender<dyn DataTypeTrait>) {
        self.output_interface = new_interface;
    }
    pub fn start(&mut self) {
        for input_interface in &self.input_interfaces {
            let data_vector = self.data_vector.clone();
            std::thread::spawn(move || {
                loop {
                    let data = input_interface.recv().unwrap();
                    data_vector.entry(data.get_id()).or_insert_with(Vec::new).push(data.clone());
                }
            });
        }
        for (id, data) in &self.data_vector {
            if (data.len() == self.input_interfaces.len()) {
                let data = data.pop().unwrap();
                self.output_interface.send(data.clone()).unwrap();
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Send, Sync)]
pub trait AlgorithmBlock {
    fn get_name(&self) -> String;
    fn get_description(&self) -> String;
    fn get_parameters(&self) -> Vec<dyn Parameter>;
    fn get_parameter(&self, name: &str) -> Option<&dyn Parameter>;
    fn process(&mut self, input: Vec<Box<dyn DataTypeTrait>>) -> Vec<Box<dyn DataTypeTrait>>;
}

pub trait ProcessingSequence<T: std::fmt::Debug + std::fmt::Display + Ord + Clone + Copy + Send + Sync> {
    fn get_name(&self) -> String;
    fn get_description(&self) -> String;
    fn get_algorithm_blocks(&self) -> Vec<Box<dyn AlgorithmBlock<T>>>;
    fn append_algorithm_block(&mut self, new_block: Box<AlgorithmBlock>);
    fn get_input_interface(&self) -> Option<Box<dyn InterfaceTrait>>;
    fn get_output_interface(&self) -> Option<Box<dyn InterfaceTrait>>;
    fn append_input_interface(&mut self, new_interface: Box<dyn InterfaceTrait>);
    fn append_output_interface(&mut self, new_interface: Box<dyn InterfaceTrait>);
    fn set_receiver(&mut self, receiver: ReceiverMultiplexer);
    fn append_sender(&mut self, sender: Sender<Box<dyn DataTypeTrait>>);
    fn start(&mut self);
    fn stop(&mut self);
}

