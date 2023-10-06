use std::net::SocketAddr;

use crate::types::constants::NUM_RCVMMSGS;
use crate::types::Result;
use tokio::net::UdpSocket;
use tracing::error;
use libc;
use std::os::unix::io::AsRawFd;
// NOTE: For Linux we can use system call from libc::recv_mmsg

/// It receives a UDP packet from a socket, and
/// returns the index of the packet in the array, the number of bytes received,
/// and the address of the sender
///
/// Arguments:
///
/// * `socket`: The UDP socket to receive from.
/// * `packets`: a mutable array of byte arrays, each of which is the size of
///   the largest packet you
/// want to receive.
#[cfg(not(target_os = "linux"))]
pub async fn recv_mmsg(
    socket: &UdpSocket,
    packets: &mut [[u8; 1280]],
) -> Result<Vec<(usize, usize, SocketAddr)>> {
    let mut received = Vec::new();

    let count = std::cmp::min(NUM_RCVMMSGS, packets.len());

    for (i, packt) in packets.iter_mut().take(count).enumerate() {
        match socket.recv_from(packt).await {
            Err(err) => {
                error!("Error occured while receiving packet: {:?}", err);
            }
            Ok((nrecv, from)) => {
                received.push((i, nrecv, from));
            }
        }
    }

    Ok(received)
}

#[cfg(target_os = "linux")]
pub async fn recv_mmsg(
    socket: &UdpSocket,
    packets: &mut [[u8; 1280]],
) -> Result<Vec<(usize, usize, SocketAddr)>> {
    const NUM_MESSAGES: usize = 10;

    let mut mmsghdrs: [libc::mmsghdr; NUM_MESSAGES] = unsafe { std::mem::zeroed() };
    let mut msghdrs: [libc::msghdr; NUM_MESSAGES] = unsafe { std::mem::zeroed() };
    let mut iovecs: [libc::iovec; NUM_MESSAGES] = unsafe { std::mem::zeroed() };
    let mut src_addresses: [libc::sockaddr_storage; NUM_MESSAGES] = unsafe { std::mem::zeroed() };

    let count = std::cmp::min(NUM_MESSAGES, packets.len());

    for i in 0..count {
        iovecs[i].iov_base = packets[i].as_mut_ptr() as *mut libc::c_void;
        iovecs[i].iov_len = packets[i].len();

        msghdrs[i].msg_name = &mut src_addresses[i] as *mut _ as *mut libc::c_void;
        msghdrs[i].msg_namelen = std::mem::size_of::<libc::sockaddr_storage>() as libc::socklen_t;

        msghdrs[i].msg_iov = &mut iovecs[i];
        msghdrs[i].msg_iovlen = 1;

        mmsghdrs[i].msg_hdr = msghdrs[i];
    }

    let received_count = unsafe {
        libc::recvmmsg(
            socket.as_raw_fd(),
            mmsghdrs.as_mut_ptr(),
            count as libc::c_uint,
            0,
            std::ptr::null_mut()
        )
    };

    if received_count == -1 {
        return Err(crate::types::DyswarmError::Io(std::io::Error::last_os_error().into()));
    }

    let mut results = Vec::new();
    for i in 0..received_count as usize {
        let len = mmsghdrs[i].msg_len as usize; // Number of bytes received

        let addr = unsafe {
            let addr_ptr = &src_addresses[i] as *const libc::sockaddr_storage;
            match (*addr_ptr).ss_family as libc::c_int {
                libc::AF_INET => {
                    let addr_in = addr_ptr as *const libc::sockaddr_in;
                    let ip = std::net::Ipv4Addr::from((*addr_in).sin_addr.s_addr.to_be_bytes());
                    let port = u16::from_be((*addr_in).sin_port);
                    SocketAddr::V4(std::net::SocketAddrV4::new(ip, port))
                },
                libc::AF_INET6 => {
                    let addr_in6 = addr_ptr as *const libc::sockaddr_in6;
                    let ip = std::net::Ipv6Addr::from((*addr_in6).sin6_addr.s6_addr);
                    let port = u16::from_be((*addr_in6).sin6_port);
                    SocketAddr::V6(std::net::SocketAddrV6::new(ip, port, 0, 0))
                },
                _ => continue, // Unknown address family
            }
        };

        results.push((i, len, addr));
    }

    Ok(results)
}

#[derive(Debug, Default)]
pub struct PacketReceiver {
    // TODO: develop further
}

impl PacketReceiver {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn run() {
        //
    }
}
