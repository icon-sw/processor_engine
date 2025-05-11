use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use crate::log::{log, LogEntry, LogLevel};
#[derive(Clone)]
pub enum PhysInterface {
    None,
    Serial,
    Ethernet,
    I2C,
    SPI,
    CAN,
    RS232,
    RS485,
}
#[derive(Clone)]
pub enum LogicalInterface {
    File,
    Log,
    Socket,
    Pipe,
    SharedMemory,
    MessageQueue,
    Signal,
}
#[derive(Clone)]
pub enum InterfaceType {
    Physical(PhysInterface),
    Logical(LogicalInterface),
}
#[derive(Clone)]
pub enum InterfaceStatus {
    Connected,
    Disconnected,
    Error,
}

#[derive(Clone)]
pub enum InterfaceMode {
    Read,
    Write,
    ReadWrite,
}
#[derive(Clone)]
pub enum InterfaceProtocol {
    Custom,
    TCP,
    UDP,
    CANopen,
    EtherCAT,
}
#[derive(Clone)]
pub enum InterfaceError {
    Timeout,
    Overflow,
    Underflow,
    FramingError,
    ParityError,
    ChecksumError,
    ProtocolError,
    WriteOnReadOnly,
    ReadOnWriteOnly,
    NotOpenIFace,
    AlreadyOpenIFace,
    GenericError,
}
impl ToString for InterfaceError {
    fn to_string(&self) -> String {
        match self {
            InterfaceError::Timeout => "Timeout".to_string(),
            InterfaceError::Overflow => "Overflow".to_string(),
            InterfaceError::Underflow => "Underflow".to_string(),
            InterfaceError::FramingError => "Framing Error".to_string(),
            InterfaceError::ParityError => "Parity Error".to_string(),
            InterfaceError::ChecksumError => "Checksum Error".to_string(),
            InterfaceError::ProtocolError => "Protocol Error".to_string(),
            InterfaceError::WriteOnReadOnly => "Write on Read Only".to_string(),
            InterfaceError::ReadOnWriteOnly => "Read on Write Only".to_string(),
            InterfaceError::NotOpenIFace => "Interface not open".to_string(),
            InterfaceError::AlreadyOpenIFace => "Interface already open".to_string(),
            InterfaceError::GenericError => "Unpredictable error".to_string(),
        }
    }
}
#[derive(Clone)]
pub enum InterfaceEvent {
    DataReceived,
    DataSent,
    ConnectionEstablished,
    ConnectionLost,
    ErrorOccurred,
}

struct BaseInterface {
    name: String,
    description: String,
    status: InterfaceStatus,
    mode: InterfaceMode,
    interface_type: InterfaceType,
    error: Option<InterfaceError>,
    event: Option<InterfaceEvent>,
}

impl BaseInterface {
    fn get_name(&self) -> String {
        self.name.clone()
    }
    fn get_description(&self) -> String {
        self.description.clone()
    }
    fn get_type(&self) -> InterfaceType {
        self.interface_type.clone()
    }
    fn get_status(&self) -> InterfaceStatus {
        self.status.clone()
    }
    fn get_mode(&self) -> InterfaceMode {
        self.mode.clone()
    }
    fn get_protocol(&self) -> InterfaceProtocol {
        InterfaceProtocol::Custom
    }
    fn get_error(&self) -> Option<InterfaceError> {
        self.error.clone()
    }
    fn get_event(&self) -> Option<InterfaceEvent> {
        self.event.clone()
    }
    fn is_log_interface(&self) -> bool {
        match self.get_type() {
            InterfaceType::Logical(LogicalInterface::Log) => true,
            _ => false,
        }
    }
    fn log_error(&mut self) {
        if self.is_log_interface() {
            return;
        }
        if let Some(error) = self.get_error() {
            log().write(LogEntry::new(
                LogLevel::ERR,
                format!("interface:{}", self.get_name()),
                format!("{}", error.to_string()),
            ));
        }
    }
}

pub trait InterfaceTrait {
    fn open(&mut self) -> Result<(), String>;
    fn close(&mut self) -> Result<(), String>;
    fn read(&mut self, buffer: &mut [u8]) -> Result<u32, String>;
    fn write(&mut self, buffer: &[u8]) -> Result<(), String>;
}

pub trait IsInterfaceManager {
    fn add_interface(&mut self, interface: Box<dyn InterfaceTrait>) -> Result<(), String>;
    fn remove_interface(&mut self, interface: &Box<dyn InterfaceTrait>) -> Result<(), String>;
    fn get_interface(&self, index: u32) -> Option<&Box<dyn InterfaceTrait>>;
    fn get_interface_count(&self) -> u32;
    fn open_all_interfaces(&mut self) -> Result<(), String>;
    fn close_all_interfaces(&mut self) -> Result<(), String>;
}
pub struct FileInterface {
    file_path: String,
    file: Option<File>,
    base_interface: BaseInterface,
}

impl FileInterface {
    pub fn new(name: String, description: String, file_path: String , mode: InterfaceMode, log_if: Option<bool>) -> Self {
        FileInterface {
            file_path,
            file: None,
            base_interface: BaseInterface {
                name,
                description,
                status: InterfaceStatus::Disconnected,
                mode,
                interface_type: {
                    if log_if.is_some() && log_if.unwrap() {
                        InterfaceType::Logical(LogicalInterface::Log)
                    } else {
                        InterfaceType::Logical(LogicalInterface::File)
                    }
                },
                error: None,
                event: None,

            },
        }
    }
}
impl InterfaceTrait for FileInterface {
    fn open(&mut self) -> Result<(), String> {
        match  self.base_interface.get_status() {
            InterfaceStatus::Connected => {
                self.base_interface.error = Some(InterfaceError::AlreadyOpenIFace);
                return Err(self.base_interface.error.clone().unwrap().to_string());
            }
            _ => {}
        }
        match self.base_interface.get_mode() {
            InterfaceMode::Read => {
                self.file = Some(File::open(&self.file_path).map_err(|e| e.to_string())?);
            }
            InterfaceMode::Write => {
                self.file = Some(File::create(&self.file_path).map_err(|e| e.to_string())?);
            }
            InterfaceMode::ReadWrite => {
                self.file = Some(OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(&self.file_path)
                    .map_err(|e| e.to_string())?);
            }
        }
        self.base_interface.status = InterfaceStatus::Connected;
        Ok(())
    }
    fn close(&mut self) -> Result<(), String> {
        match self.base_interface.status {
            InterfaceStatus::Disconnected => {
                self.base_interface.error = Some(InterfaceError::NotOpenIFace);
                return Err(self.base_interface.error.clone().unwrap().to_string());
            }
            _ => {}
        }
        if self.file.is_some() {
            if let Some(file) = self.file.take() {
                file.sync_all().map_err(|e| e.to_string())?;
            }
            self.base_interface.status = InterfaceStatus::Disconnected;
            Ok(())
        } else {
            self.base_interface.error = Some(InterfaceError::NotOpenIFace);
            return Err(self.base_interface.error.clone().unwrap().to_string());
        }
    }

    fn read(&mut self, buffer: &mut [u8]) -> Result<u32, String> {
        match self.base_interface.mode {
            InterfaceMode::Write => {
                self.base_interface.error = Some(InterfaceError::WriteOnReadOnly);
                return Err(self.base_interface.error.clone().unwrap().to_string());
            }
            _ => {}
        }
        match self.base_interface.status {
            InterfaceStatus::Connected => {
                self.base_interface.error = None;
                if let Some(file) = self.file.as_mut() {
                    let bytes_read = file.read(buffer).map_err(|e| e.to_string())?;
                    return Ok(bytes_read as u32);
                } else {
                    self.base_interface.error = Some(InterfaceError::GenericError);
                    return Err(self.base_interface.error.clone().unwrap().to_string());
                }
            }
            _ => {
                self.base_interface.error = Some(InterfaceError::NotOpenIFace);
                return Err(self.base_interface.error.clone().unwrap().to_string());
            }
        }
    }

    fn write(&mut self, buffer: &[u8]) -> Result<(), String> {
        match self.base_interface.mode {
            InterfaceMode::Read => {
                self.base_interface.error = Some(InterfaceError::ReadOnWriteOnly);
                return Err(self.base_interface.error.clone().unwrap().to_string());
            }
            _ => {}
        }
        match self.base_interface.status {
            InterfaceStatus::Connected => {
                self.base_interface.error = None;
                if let Some(file) = self.file.as_mut() {
                    file.write_all(buffer).map_err(|e| e.to_string())?;
                    return Ok(());
                }
                else {
                    self.base_interface.error = Some(InterfaceError::GenericError);
                    return Err(self.base_interface.error.clone().unwrap().to_string());
                }
            }
            _ => {
                self.base_interface.error = Some(InterfaceError::NotOpenIFace);
                return Err(self.base_interface.error.clone().unwrap().to_string());
            }
        }
    }
}

