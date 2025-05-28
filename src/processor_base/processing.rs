use std::collections::HashMap;
use std::thread;
use std::sync::{Arc, Mutex};

use spmc::{Sender, Receiver}; // Assuming you have a crate for single-producer, multi-consumer channels

use crate::interfaces::InterfaceTrait;
use super::parameter::ParameterTrait;

#[derive(Clone)]
pub struct DataProcessor {
    ifcode: u64,
    id: u64,
    timestamp_sec: u64,
    timestamp_nsec: u64,
    data_size: u64,
    data: Vec<u8>,
}

impl DataProcessor {
    pub fn new(ifcode: u64, id: u64, timestamp_sec: u64, timestamp_nsec: u64, data_size: u64, data: Vec<u8>) -> Self {
        DataProcessor {
            ifcode, // Default value, can be set later if needed
            id,
            timestamp_sec,
            timestamp_nsec,
            data_size,
            data,
        }
    }
    pub fn ifcode(&self) -> u64 {
        self.ifcode
    }

    pub fn id(&self) -> u64 {
        self.id
    }
    pub fn timestamp(&self) -> f64 {
        self.timestamp_sec as f64 + self.timestamp_nsec as f64 * 1e-9
    }
    pub fn data_size(&self) -> u64 {
        self.data_size
    }
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

impl std::fmt::Debug for DataProcessor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "DataProcessor {{ id: {}, timestamp: {:.9}, data_size: {} }}",
            self.id,
            self.timestamp(),
            self.data_size
        )
    }
}
impl std::fmt::Display for DataProcessor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "DataProcessor ID: {}, Timestamp: {:.9}, Data Size: {} bytes",
            self.id,
            self.timestamp(),
            self.data_size
        )
    }
}

pub struct ReceiverMultiplexer {
    input_data: Vec<Receiver<DataProcessor>>,
    output_data: Option<Sender<Vec<DataProcessor>>>,
    data_map: Arc<Mutex<HashMap<u64, Vec<DataProcessor>>>>,
}

impl ReceiverMultiplexer {
    pub fn new(input_data: Vec<Receiver<DataProcessor>>, output_data: Option<Sender<Vec<DataProcessor>>>) -> Self {
        ReceiverMultiplexer {
            input_data,
            output_data,
            data_map: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn add_input_receiver(&mut self, receiver: Receiver<DataProcessor>) {
        self.input_data.push(receiver);
    }
    pub fn set_output_sender(&mut self, sender: Sender<Vec<DataProcessor>>) {
        self.output_data = Some(sender);
    }

    pub fn process(&'static mut self) {
        for receiver in &self.input_data {
            let data_map = Arc::clone(&self.data_map);
            thread::spawn(move ||
                loop {
                    if let Ok(data) = receiver.try_recv() {
                    // Process the data here
                        let mut map = data_map.lock().unwrap();
                        let entry = map.entry(data.id()).or_insert_with(Vec::new);
                        entry.push(data);
                    }
                }
            );
        }

        if let Some(sender) = &mut self.output_data {
            let sender_size = self.input_data.len();
            loop {
                let mut map = self.data_map.lock().unwrap();
                let mut send= false;
                let mut last_send = 0;
                for (id, data_list) in map.clone().into_iter() {
                    if data_list.len() == sender_size {
                        sender.send(data_list.clone()).unwrap();
                        map.remove(&id); // Remove the entry after sending
                        send = true;
                        last_send = id; // Keep track of the last sent ID
                    } else if id < last_send {
                        map.remove(&id); // Remove entries that are older than the last sent ID
                    } else if send {
                        // If we have sent data, we can break to avoid holding the lock too long
                        break;
                    }
                }
                // Clear the map after sending
                drop(map); // Explicitly drop the lock before sleeping
            }
        }
    }
}

pub trait AlgorithmBlock {
    fn get_name(&self) -> String;
    fn get_description(&self) -> String;
    fn get_parameters(&self) -> Vec<impl ParameterTrait>;
    fn get_parameter(&self, name: &str) -> Option<&impl ParameterTrait>;
    fn process(&mut self, input: Vec<Box<DataProcessor>>) -> Vec<Box<DataProcessor>>;
}

pub trait ProcessingSequence<T: std::fmt::Debug + std::fmt::Display + Ord + Clone + Copy + Send + Sync> {
    fn get_name(&self) -> String;
    fn get_description(&self) -> String;
    fn get_algorithm_blocks(&self) -> Vec<Box<impl AlgorithmBlock>>;
    fn append_algorithm_block(&mut self, new_block: Box<impl AlgorithmBlock>);
    fn get_input_interface(&self) -> Option<Box<impl InterfaceTrait>>;
    fn get_output_interface(&self) -> Option<Box<impl InterfaceTrait>>;
    fn append_input_interface(&mut self, new_interface: Box<impl InterfaceTrait>);
    fn append_output_interface(&mut self, new_interface: Box<impl InterfaceTrait>);
    fn append_receiver(&mut self, receiver: ReceiverMultiplexer);
    fn append_sender(&mut self, sender: Sender<DataProcessor>);
    fn start(&mut self);
    fn stop(&mut self);
}
