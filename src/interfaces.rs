use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

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
    Idle,
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

pub trait IsInterface {
    fn get_type(&self) -> InterfaceType;
    fn get_status(&self) -> InterfaceStatus;
    fn get_mode(&self) -> InterfaceMode;
    fn get_protocol(&self) -> InterfaceProtocol;
    fn get_error(&self) -> Option<InterfaceError>;
    fn get_event(&self) -> Option<InterfaceEvent>;
    fn open(&mut self) -> Result<(), String>;
    fn close(&mut self) -> Result<(), String>;
    fn read(&mut self, buffer: &mut [u8]) -> Result<u32, String>;
    fn write(&mut self, buffer: &[u8]) -> Result<(), String>;
}
pub trait IsInterfaceManager {
    fn add_interface(&mut self, interface: Box<dyn IsInterface>) -> Result<(), String>;
    fn remove_interface(&mut self, interface: &Box<dyn IsInterface>) -> Result<(), String>;
    fn get_interface(&self, index: u32) -> Option<&Box<dyn IsInterface>>;
    fn get_interface_count(&self) -> u32;
    fn open_all_interfaces(&mut self) -> Result<(), String>;
    fn close_all_interfaces(&mut self) -> Result<(), String>;
}
pub struct FileInterface {
    name: String,
    description: String,
    file_path: String,
    interface_type: InterfaceType,
    status: InterfaceStatus,
    mode: InterfaceMode,
    protocol: InterfaceProtocol,
    error: Option<InterfaceError>,
    event: Option<InterfaceEvent>,
    file: Option<File>,
}

impl FileInterface {
    pub fn new(name: String, description: String, file_path: String , mode: InterfaceMode) -> Self {
        FileInterface {
            name,
            description,
            file_path,
            interface_type: InterfaceType::Logical(LogicalInterface::File),
            status: InterfaceStatus::Disconnected,
            mode,
            protocol: InterfaceProtocol::Custom,
            error: None,
            event: None,
            file: None,
        }
    }
}
impl IsInterface for FileInterface {
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
        self.protocol.clone()
    }
    fn get_error(&self) -> Option<InterfaceError> {
        self.error.clone()
    }
    fn get_event(&self) -> Option<InterfaceEvent> {
        self.event.clone()
    }
    fn open(&mut self) -> Result<(), String> {
        match  self.status {
            InterfaceStatus::Connected => {
                self.error = Some(InterfaceError::AlreadyOpenIFace);
                return Err(self.error.clone().unwrap().to_string());
            }
            _ => {}
        }
        match self.mode {
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
        self.status = InterfaceStatus::Connected;
        Ok(())
    }
    fn close(&mut self) -> Result<(), String> {
        match self.status {
            InterfaceStatus::Disconnected => {
                self.error = Some(InterfaceError::NotOpenIFace);
                return Err(self.error.clone().unwrap().to_string());
            }
            _ => {}
        }
        if self.file.is_some() {
            if let Some(file) = self.file.take() {
                file.sync_all().map_err(|e| e.to_string())?;
            }
            self.status = InterfaceStatus::Disconnected;
            Ok(())
        } else {
            self.error = Some(InterfaceError::NotOpenIFace);
            return Err(self.error.clone().unwrap().to_string());
        }
    }

    fn read(&mut self, buffer: &mut [u8]) -> Result<u32, String> {
        match self.mode {
            InterfaceMode::Write => {
                self.error = Some(InterfaceError::WriteOnReadOnly);
                return Err(self.error.clone().unwrap().to_string());
            }
            _ => {}
        }
        match self.status {
            InterfaceStatus::Connected => {
                self.error = None;
                if let Some(file) = self.file.as_mut() {
                    let bytes_read = file.read(buffer).map_err(|e| e.to_string())?;
                    return Ok(bytes_read as u32);
                } else {
                    self.error = Some(InterfaceError::GenericError);
                    return Err(self.error.clone().unwrap().to_string());
                }
            }
            _ => {
                self.error = Some(InterfaceError::NotOpenIFace);
                return Err(self.error.clone().unwrap().to_string());
            }
        }
    }

    fn write(&mut self, buffer: &[u8]) -> Result<(), String> {
        match self.mode {
            InterfaceMode::Read => {
                self.error = Some(InterfaceError::ReadOnWriteOnly);
                return Err(self.error.clone().unwrap().to_string());
            }
            _ => {}
        }
        match self.status {
            InterfaceStatus::Connected => {
                self.error = None;
                if let Some(file) = self.file.as_mut() {
                    file.write_all(buffer).map_err(|e| e.to_string())?;
                    return Ok(());
                }
                else {
                    self.error = Some(InterfaceError::GenericError);
                    return Err(self.error.clone().unwrap().to_string());
                }
            }
            _ => {
                self.error = Some(InterfaceError::NotOpenIFace);
                return Err(self.error.clone().unwrap().to_string());
            }
        }
    }
}

