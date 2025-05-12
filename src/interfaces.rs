use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::net::{UdpSocket, IpAddr, Ipv4Addr, Ipv6Addr};
use std::str::FromStr;
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
    Socket,
    Pipe,
    SharedMemory,
    MessageQueue,
    Signal,
}
#[derive(Clone)]
pub struct InterfaceType {
    phys: PhysInterface,
    logic: LogicalInterface,
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
    Raw,
    TcpIp,
    UdpIp,
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
    NotValidSocketAddr,
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
            InterfaceError::NotValidSocketAddr => "Not valid socket address".to_string(),
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
    interface_protocol: InterfaceProtocol,
    log_interface: bool,
    error: Option<InterfaceError>,
    event: Option<InterfaceEvent>,
}

impl BaseInterface {
    fn new(
        name: String,
        description: String,
        interface_type: InterfaceType,
        mode: InterfaceMode,
        interface_protocol: InterfaceProtocol,
        log_if: Option<bool>,
    ) -> Self {
        BaseInterface {
            name,
            description,
            status: InterfaceStatus::Disconnected,
            mode,
            interface_type,
            interface_protocol,
            log_interface: {
                if log_if.is_some() {
                    log_if.unwrap()
                } else {
                    false
                }
            },
            error: None,
            event: None,
        }
    }
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
        self.interface_protocol.clone()
    }
    fn get_error(&self) -> Option<InterfaceError> {
        self.error.clone()
    }
    fn get_event(&self) -> Option<InterfaceEvent> {
        self.event.clone()
    }
    fn is_log_interface(&self) -> bool {
        self.log_interface.clone()
    }

    fn set_error(&mut self, error: InterfaceError) {
        self.error = Some(error);
        self.log_error();
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
            base_interface: BaseInterface::new(name,
                                            description,
                                            InterfaceType{phys: PhysInterface::None, logic: LogicalInterface::File},
                                            mode,
                                            InterfaceProtocol::Raw,
                                            log_if),
        }
    }
}
impl InterfaceTrait for FileInterface {
    fn open(&mut self) -> Result<(), String> {
        match  self.base_interface.get_status() {
            InterfaceStatus::Connected => {
                self.base_interface.set_error(InterfaceError::AlreadyOpenIFace);
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
                self.base_interface.set_error(InterfaceError::NotOpenIFace);
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
            self.base_interface.set_error(InterfaceError::NotOpenIFace);
            return Err(self.base_interface.error.clone().unwrap().to_string());
        }
    }

    fn read(&mut self, buffer: &mut [u8]) -> Result<u32, String> {
        match self.base_interface.mode {
            InterfaceMode::Write => {
                self.base_interface.set_error(InterfaceError::WriteOnReadOnly);
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
                    self.base_interface.set_error(InterfaceError::GenericError);
                    return Err(self.base_interface.error.clone().unwrap().to_string());
                }
            }
            _ => {
                self.base_interface.set_error(InterfaceError::NotOpenIFace);
                return Err(self.base_interface.error.clone().unwrap().to_string());
            }
        }
    }

    fn write(&mut self, buffer: &[u8]) -> Result<(), String> {
        match self.base_interface.mode {
            InterfaceMode::Read => {
                self.base_interface.set_error(InterfaceError::ReadOnWriteOnly);
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
                    self.base_interface.set_error(InterfaceError::GenericError);
                    return Err(self.base_interface.error.clone().unwrap().to_string());
                }
            }
            _ => {
                self.base_interface.set_error(InterfaceError::NotOpenIFace);
                return Err(self.base_interface.error.clone().unwrap().to_string());
            }
        }
    }
}

pub struct UDPInterface {
    ip_address: String,
    port: u16,
    socket: Option<UdpSocket>,
    remote_addr: String,
    remote_port: u16,
    remote_socket_addr: Option<std::net::SocketAddr>,
    multicast: bool,
    base_interface: BaseInterface,
}
impl UDPInterface {
    pub fn new(name: String, description: String, ip_address: String, port: u16, log_if: Option<bool>) -> Self {
        if format!("{}:{}", ip_address, port).parse::<std::net::SocketAddr>().is_err() {
            panic!("Invalid IP address or port");
        }
        UDPInterface {
            ip_address,
            port,
            remote_addr: "".to_string(),
            remote_port: 0,
            socket: None,
            remote_socket_addr: None,
            multicast: false,
            base_interface: BaseInterface::new(name,
                                            description,
                                            InterfaceType{phys: PhysInterface::Ethernet, logic: LogicalInterface::Socket},
                                            InterfaceMode::ReadWrite,
                                            InterfaceProtocol::UdpIp,
                                            log_if),
        }
    }
    pub fn append_remote_addr(&mut self, remote_ip: String, remote_port: u16) {
        match self.base_interface.get_mode() {
            InterfaceMode::Read => {
                self.base_interface.set_error(InterfaceError::ReadOnWriteOnly);
                return;
            }
            _ => {}
        }
        let socket_addr = format!("{}:{}", remote_ip, remote_port);
        if socket_addr.parse::<std::net::SocketAddr>().is_err() {
            self.base_interface.set_error(InterfaceError::NotValidSocketAddr);
            return;
        }

        self.remote_addr = remote_ip;
        self.remote_port = remote_port;
    }
}

impl InterfaceTrait for UDPInterface {
    fn open(&mut self) -> Result<(), String> {
        match  self.base_interface.get_status() {
            InterfaceStatus::Connected => {
                self.base_interface.set_error(InterfaceError::AlreadyOpenIFace);
                return Err(self.base_interface.error.clone().unwrap().to_string());
            }
            _ => {}
        }
        // Implement UDP connection opening logic here
        self.socket = Some(UdpSocket::bind((self.ip_address.as_str(), self.port)).map_err(|e| e.to_string())?);
        let remote_ip_addr = self.remote_addr.as_str();
        match IpAddr::from_str(remote_ip_addr) {
            Ok(ip_addr) => {
                if ip_addr.is_multicast() {
                    let socket = self.socket.as_ref().unwrap();
                    if ip_addr.is_ipv4() {
                        let ipv4 = Ipv4Addr::from_str(remote_ip_addr).map_err(|e| e.to_string())?;
                        socket.set_multicast_loop_v4(true).map_err(|e| e.to_string())?;
                        socket.join_multicast_v4(&ipv4, &Ipv4Addr::new(0, 0, 0, 0))
                                .map_err(|e| e.to_string())?;
                    }
                    else {
                        let ipv6 = Ipv6Addr::from_str(remote_ip_addr).map_err(|e| e.to_string())?;
                        socket.set_multicast_loop_v6(true).map_err(|e| e.to_string())?;
                        socket.join_multicast_v6(&ipv6, 0).map_err(|e| e.to_string())?;
                    }
                }
                self.remote_socket_addr = Some(format!("{}:{}", remote_ip_addr, self.remote_port)
                                            .parse::<std::net::SocketAddr>().map_err(|e| e.to_string())?);
            }
            Err(_) => {}
        }

        self.base_interface.status = InterfaceStatus::Connected;
        Ok(())
    }

    fn close(&mut self) -> Result<(), String> {
        match self.base_interface.status {
            InterfaceStatus::Disconnected => {
                self.base_interface.set_error(InterfaceError::NotOpenIFace);
                return Err(self.base_interface.error.clone().unwrap().to_string());
            }
            _ => {}
        }
        // Implement UDP connection closing logic here
        if let Some(ref socket) = self.socket {
            self.base_interface.status = InterfaceStatus::Disconnected;
            if self.multicast {
                if let Some(ref remote_addr) = self.remote_socket_addr {
                    if remote_addr.is_ipv6() {
                        socket.leave_multicast_v6(&Ipv6Addr::from_str(&self.remote_addr).unwrap(), 0)
                            .map_err(|e| e.to_string())?;
                    }
                    else {
                        socket.leave_multicast_v4(&Ipv4Addr::from_str(&self.remote_addr).unwrap(), &Ipv4Addr::new(0, 0, 0, 0))
                            .map_err(|e| e.to_string())?;
                    }
                }
            }
            Ok(())
        }
        else {
            self.base_interface.set_error(InterfaceError::NotOpenIFace);
            return Err(self.base_interface.error.clone().unwrap().to_string());
        }

    }

    fn read(&mut self, buffer: &mut [u8]) -> Result<u32, String> {
        // Implement UDP reading logic here
        match self.base_interface.get_mode() {
            InterfaceMode::Write => {
                self.base_interface.set_error(InterfaceError::WriteOnReadOnly);
                return Err(self.base_interface.error.clone().unwrap().to_string());
            }
            _ => {}
        }
        if self.socket.is_some() {
            if let Some(ref socket) = self.socket {
                let (bytes_read, _) = socket.recv_from(buffer).map_err(|e| e.to_string())?;
                return Ok(bytes_read as u32);
            }
            else {
                self.base_interface.set_error(InterfaceError::GenericError);
                return Err(self.base_interface.error.clone().unwrap().to_string());
            }
        }
        else {
            self.base_interface.set_error(InterfaceError::NotOpenIFace);
            return Err(self.base_interface.error.clone().unwrap().to_string());
        }
    }

    fn write(&mut self, buffer: &[u8]) -> Result<(), String> {
        // Implement UDP writing logic here
        match self.base_interface.get_mode() {
            InterfaceMode::Read => {
                self.base_interface.set_error(InterfaceError::ReadOnWriteOnly);
                return Err(self.base_interface.error.clone().unwrap().to_string());
            }
            _ => {}
        }
        if let Some(ref remote_addr) = self.remote_socket_addr {
            if let Some(ref socket) = self.socket {
                socket.send_to(buffer, remote_addr).map_err(|e| e.to_string())?;
                return Ok(());
            }
            else {
                self.base_interface.set_error(InterfaceError::NotOpenIFace);
                return Err(self.base_interface.error.clone().unwrap().to_string());
            }
        }
        else {
            self.base_interface.set_error(InterfaceError::GenericError);
            return Err(self.base_interface.error.clone().unwrap().to_string());
        }
    }
}
