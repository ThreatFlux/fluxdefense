use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::sync::Arc;
use anyhow::{Result, anyhow};
use tracing::{info, warn, error, debug};
use libc::{self, c_int, sockaddr_nl};
use std::mem;
use std::os::unix::io::{AsRawFd, RawFd};

// Netlink constants
const NETLINK_INET_DIAG: c_int = 4;
const SOCK_DIAG_BY_FAMILY: c_int = 20;

// Socket families
const AF_INET: u8 = 2;
const AF_INET6: u8 = 10;

// TCP states
const TCP_ESTABLISHED: u8 = 1;
const TCP_SYN_SENT: u8 = 2;
const TCP_SYN_RECV: u8 = 3;
const TCP_FIN_WAIT1: u8 = 4;
const TCP_FIN_WAIT2: u8 = 5;
const TCP_TIME_WAIT: u8 = 6;
const TCP_CLOSE: u8 = 7;
const TCP_CLOSE_WAIT: u8 = 8;
const TCP_LAST_ACK: u8 = 9;
const TCP_LISTEN: u8 = 10;
const TCP_CLOSING: u8 = 11;

#[repr(C)]
struct NetlinkMessageHeader {
    nlmsg_len: u32,
    nlmsg_type: u16,
    nlmsg_flags: u16,
    nlmsg_seq: u32,
    nlmsg_pid: u32,
}

#[repr(C)]
struct InetDiagReqV2 {
    sdiag_family: u8,
    sdiag_protocol: u8,
    idiag_ext: u8,
    pad: u8,
    idiag_states: u32,
    id: InetDiagSockId,
}

#[repr(C)]
struct InetDiagSockId {
    idiag_sport: u16,
    idiag_dport: u16,
    idiag_src: [u32; 4],
    idiag_dst: [u32; 4],
    idiag_if: u32,
    idiag_cookie: [u32; 2],
}

#[repr(C)]
struct InetDiagMsg {
    idiag_family: u8,
    idiag_state: u8,
    idiag_timer: u8,
    idiag_retrans: u8,
    id: InetDiagSockId,
    idiag_expires: u32,
    idiag_rqueue: u32,
    idiag_wqueue: u32,
    idiag_uid: u32,
    idiag_inode: u32,
}

pub struct NetlinkMonitor {
    socket: RawFd,
    running: bool,
}

impl NetlinkMonitor {
    pub fn new() -> Result<Self> {
        info!("Initializing netlink monitor for network connections");
        
        // Create netlink socket
        let socket = unsafe {
            libc::socket(libc::AF_NETLINK, libc::SOCK_RAW, NETLINK_INET_DIAG)
        };
        
        if socket < 0 {
            let err = std::io::Error::last_os_error();
            return Err(anyhow!("Failed to create netlink socket: {}", err));
        }
        
        // Bind to netlink
        let mut addr: sockaddr_nl = unsafe { mem::zeroed() };
        addr.nl_family = libc::AF_NETLINK as u16;
        addr.nl_pid = 0; // Let kernel assign
        addr.nl_groups = 0;
        
        let ret = unsafe {
            libc::bind(
                socket,
                &addr as *const _ as *const libc::sockaddr,
                mem::size_of::<sockaddr_nl>() as u32
            )
        };
        
        if ret < 0 {
            unsafe { libc::close(socket) };
            let err = std::io::Error::last_os_error();
            return Err(anyhow!("Failed to bind netlink socket: {}", err));
        }
        
        info!("Netlink monitor initialized successfully");
        
        Ok(Self {
            socket,
            running: false,
        })
    }
    
    pub fn start_monitoring(&mut self) -> Result<()> {
        if self.running {
            return Ok(());
        }
        
        self.running = true;
        info!("Netlink monitoring started");
        Ok(())
    }
    
    pub fn get_tcp_connections(&self) -> Result<Vec<NetworkConnection>> {
        let mut connections = Vec::new();
        
        // Query IPv4 TCP connections
        connections.extend(self.query_connections(AF_INET, libc::IPPROTO_TCP as u8)?);
        
        // Query IPv6 TCP connections
        connections.extend(self.query_connections(AF_INET6, libc::IPPROTO_TCP as u8)?);
        
        Ok(connections)
    }
    
    pub fn get_udp_connections(&self) -> Result<Vec<NetworkConnection>> {
        let mut connections = Vec::new();
        
        // Query IPv4 UDP connections
        connections.extend(self.query_connections(AF_INET, libc::IPPROTO_UDP as u8)?);
        
        // Query IPv6 UDP connections
        connections.extend(self.query_connections(AF_INET6, libc::IPPROTO_UDP as u8)?);
        
        Ok(connections)
    }
    
    fn query_connections(&self, family: u8, protocol: u8) -> Result<Vec<NetworkConnection>> {
        let mut connections = Vec::new();
        
        // Build request
        let mut req = InetDiagReqV2 {
            sdiag_family: family,
            sdiag_protocol: protocol,
            idiag_ext: 0,
            pad: 0,
            idiag_states: 0xffffffff, // All states
            id: unsafe { mem::zeroed() },
        };
        
        let nlh = NetlinkMessageHeader {
            nlmsg_len: (mem::size_of::<NetlinkMessageHeader>() + mem::size_of::<InetDiagReqV2>()) as u32,
            nlmsg_type: SOCK_DIAG_BY_FAMILY as u16,
            nlmsg_flags: (libc::NLM_F_REQUEST | libc::NLM_F_DUMP) as u16,
            nlmsg_seq: 1,
            nlmsg_pid: 0,
        };
        
        // Send request
        let mut buffer = Vec::new();
        buffer.extend_from_slice(unsafe {
            std::slice::from_raw_parts(&nlh as *const _ as *const u8, mem::size_of::<NetlinkMessageHeader>())
        });
        buffer.extend_from_slice(unsafe {
            std::slice::from_raw_parts(&req as *const _ as *const u8, mem::size_of::<InetDiagReqV2>())
        });
        
        let ret = unsafe {
            libc::send(self.socket, buffer.as_ptr() as *const libc::c_void, buffer.len(), 0)
        };
        
        if ret < 0 {
            let err = std::io::Error::last_os_error();
            return Err(anyhow!("Failed to send netlink request: {}", err));
        }
        
        // Read responses
        let mut recv_buffer = vec![0u8; 8192];
        loop {
            let bytes_read = unsafe {
                libc::recv(self.socket, recv_buffer.as_mut_ptr() as *mut libc::c_void, recv_buffer.len(), 0)
            };
            
            if bytes_read < 0 {
                let err = std::io::Error::last_os_error();
                if err.kind() == std::io::ErrorKind::WouldBlock {
                    break;
                }
                return Err(anyhow!("Failed to receive netlink response: {}", err));
            }
            
            if bytes_read == 0 {
                break;
            }
            
            // Parse messages
            let mut offset = 0;
            while offset < bytes_read as usize {
                if offset + mem::size_of::<NetlinkMessageHeader>() > bytes_read as usize {
                    break;
                }
                
                let nlh = unsafe {
                    &*(recv_buffer.as_ptr().add(offset) as *const NetlinkMessageHeader)
                };
                
                if nlh.nlmsg_type == libc::NLMSG_DONE as u16 {
                    return Ok(connections);
                }
                
                if nlh.nlmsg_type == libc::NLMSG_ERROR as u16 {
                    return Err(anyhow!("Netlink error response"));
                }
                
                if nlh.nlmsg_len as usize > recv_buffer.len() - offset {
                    break;
                }
                
                // Parse inet_diag_msg
                if offset + mem::size_of::<NetlinkMessageHeader>() + mem::size_of::<InetDiagMsg>() <= bytes_read as usize {
                    let msg = unsafe {
                        &*(recv_buffer.as_ptr().add(offset + mem::size_of::<NetlinkMessageHeader>()) as *const InetDiagMsg)
                    };
                    
                    let conn = self.parse_connection(msg)?;
                    connections.push(conn);
                }
                
                offset += nlh.nlmsg_len as usize;
                // Align to 4-byte boundary
                offset = (offset + 3) & !3;
            }
        }
        
        Ok(connections)
    }
    
    fn parse_connection(&self, msg: &InetDiagMsg) -> Result<NetworkConnection> {
        let local_addr = match msg.idiag_family {
            AF_INET => {
                let addr = Ipv4Addr::from(msg.id.idiag_src[0].to_be());
                IpAddr::V4(addr)
            }
            AF_INET6 => {
                let addr = Ipv6Addr::from([
                    (msg.id.idiag_src[0] >> 16) as u16,
                    (msg.id.idiag_src[0] & 0xffff) as u16,
                    (msg.id.idiag_src[1] >> 16) as u16,
                    (msg.id.idiag_src[1] & 0xffff) as u16,
                    (msg.id.idiag_src[2] >> 16) as u16,
                    (msg.id.idiag_src[2] & 0xffff) as u16,
                    (msg.id.idiag_src[3] >> 16) as u16,
                    (msg.id.idiag_src[3] & 0xffff) as u16,
                ]);
                IpAddr::V6(addr)
            }
            _ => return Err(anyhow!("Unknown address family")),
        };
        
        let remote_addr = match msg.idiag_family {
            AF_INET => {
                let addr = Ipv4Addr::from(msg.id.idiag_dst[0].to_be());
                IpAddr::V4(addr)
            }
            AF_INET6 => {
                let addr = Ipv6Addr::from([
                    (msg.id.idiag_dst[0] >> 16) as u16,
                    (msg.id.idiag_dst[0] & 0xffff) as u16,
                    (msg.id.idiag_dst[1] >> 16) as u16,
                    (msg.id.idiag_dst[1] & 0xffff) as u16,
                    (msg.id.idiag_dst[2] >> 16) as u16,
                    (msg.id.idiag_dst[2] & 0xffff) as u16,
                    (msg.id.idiag_dst[3] >> 16) as u16,
                    (msg.id.idiag_dst[3] & 0xffff) as u16,
                ]);
                IpAddr::V6(addr)
            }
            _ => return Err(anyhow!("Unknown address family")),
        };
        
        Ok(NetworkConnection {
            local_addr,
            local_port: msg.id.idiag_sport.to_be(),
            remote_addr,
            remote_port: msg.id.idiag_dport.to_be(),
            state: ConnectionState::from_tcp_state(msg.idiag_state),
            uid: msg.idiag_uid,
            inode: msg.idiag_inode,
        })
    }
    
    pub fn stop(&mut self) -> Result<()> {
        if self.running {
            info!("Stopping netlink monitoring");
            self.running = false;
        }
        Ok(())
    }
}

impl Drop for NetlinkMonitor {
    fn drop(&mut self) {
        if self.socket >= 0 {
            unsafe { libc::close(self.socket) };
        }
    }
}

#[derive(Debug, Clone)]
pub struct NetworkConnection {
    pub local_addr: IpAddr,
    pub local_port: u16,
    pub remote_addr: IpAddr,
    pub remote_port: u16,
    pub state: ConnectionState,
    pub uid: u32,
    pub inode: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Established,
    SynSent,
    SynRecv,
    FinWait1,
    FinWait2,
    TimeWait,
    Close,
    CloseWait,
    LastAck,
    Listen,
    Closing,
    Unknown,
}

impl ConnectionState {
    fn from_tcp_state(state: u8) -> Self {
        match state {
            TCP_ESTABLISHED => ConnectionState::Established,
            TCP_SYN_SENT => ConnectionState::SynSent,
            TCP_SYN_RECV => ConnectionState::SynRecv,
            TCP_FIN_WAIT1 => ConnectionState::FinWait1,
            TCP_FIN_WAIT2 => ConnectionState::FinWait2,
            TCP_TIME_WAIT => ConnectionState::TimeWait,
            TCP_CLOSE => ConnectionState::Close,
            TCP_CLOSE_WAIT => ConnectionState::CloseWait,
            TCP_LAST_ACK => ConnectionState::LastAck,
            TCP_LISTEN => ConnectionState::Listen,
            TCP_CLOSING => ConnectionState::Closing,
            _ => ConnectionState::Unknown,
        }
    }
}