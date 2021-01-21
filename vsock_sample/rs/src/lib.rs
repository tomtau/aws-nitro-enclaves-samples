pub mod command_parser;
pub mod protocol_helpers;
pub mod utils;

use command_parser::{ClientArgs, ServerArgs};
use protocol_helpers::{recv_loop, recv_u64, send_loop, send_u64};

use nix::sys::socket::listen as listen_vsock;
use nix::sys::socket::{accept, bind, connect, shutdown, socket};
use nix::sys::socket::{AddressFamily, Shutdown, SockAddr, SockFlag, SockType};
use nix::unistd::close;
use std::convert::TryInto;
use std::io::{Read, Write};
use std::os::unix::io::{AsRawFd, RawFd};
use vsock::{VsockListener, VsockStream};

const VSOCK_PROXY_CID: u32 = 3;
const BUF_MAX_LEN: usize = 8192;
// Maximum number of outstanding connections in the socket's
// listen queue
const BACKLOG: usize = 128;
// Maximum number of connection attempts
const MAX_CONNECTION_ATTEMPTS: usize = 5;
const BUFF_SIZE: usize = 8192;

/// Initiate a connection on an AF_VSOCK socket
fn vsock_connect(port: u32) -> Result<VsockStream, String> {
    let sockaddr = SockAddr::new_vsock(VSOCK_PROXY_CID, port);
    let mut err_msg = String::new();

    for i in 0..MAX_CONNECTION_ATTEMPTS {
        let vsocket = VsockStream::connect(&sockaddr);
        match vsocket {
            Ok(v) => return Ok(v),
            Err(e) => err_msg = format!("Failed to connect: {}", e),
        }

        // Exponentially backoff before retrying to connect to the socket
        std::thread::sleep(std::time::Duration::from_secs(1 << i));
    }

    Err(err_msg)
}

/// Send 'Hello, world!' to the server
pub fn client(args: ClientArgs) -> Result<(), String> {
    let mut vsocket1 = vsock_connect(args.port1)?;
    let mut vsocket2 = vsock_connect(args.port2)?;

    let mut buffer = [0u8; BUFF_SIZE];

    let nbytes = vsocket2.read(&mut buffer);
    let nbytes = match nbytes {
        Err(_) => 0,
        Ok(n) => n,
    };

    if nbytes > 0 {
        println!("vsock2 read: {:02X?}", &buffer[..nbytes]);
    }

    loop {
        let nbytes = vsocket1.read(&mut buffer);
        let nbytes = match nbytes {
            Err(_) => 0,
            Ok(n) => n,
        };

        if nbytes > 0 {
            println!("vsock1 read: {:02X?}", &buffer[..nbytes]);
            dbg!(vsocket2.write_all(&[1, 1, 1, 1]));
            dbg!(vsocket1.write_all(&[2, 2, 2, 2, 2]));
        } else {
            std::thread::sleep(std::time::Duration::new(1, 0));
        };
    }
    Ok(())
}

/// Accept connections on a certain port and print
/// the received data
pub fn server(args: ServerArgs) -> Result<(), String> {
    let port2 = args.port2;
    std::thread::spawn(move || {
        let sockaddr = SockAddr::new_vsock(VSOCK_PROXY_CID, port2);
        dbg!(port2);
        loop {
            match VsockListener::bind(&sockaddr) {
                Ok(listener) => {
                    println!("port2 bound listener");
                    for conn in listener.incoming() {
                        if let Ok(mut stream) = conn {
                            println!("got conn on port2");
                            dbg!(stream.write_all(&[0, 0, 0]));

                            let mut buffer = [0u8; BUFF_SIZE];

                            loop {
                                let nbytes = stream.read(&mut buffer);
                                let nbytes = match nbytes {
                                    Err(_) => 0,
                                    Ok(n) => n,
                                };

                                if nbytes > 0 {
                                    println!("port2 read: {:02X?}", &buffer[..nbytes]);
                                } else {
                                    std::thread::sleep(std::time::Duration::new(1, 0));
                                };
                            }
                        } else {
                            println!("error getting conn on port2");
                        }
                    }
                }
                Err(e) => {
                    println!("port2 bound listener failed: {:?}", e);
                    std::thread::sleep(std::time::Duration::new(1, 0));
                }
            }
        }
    });
    let sockaddr = SockAddr::new_vsock(VSOCK_PROXY_CID, args.port1);
    dbg!(args.port1);
    if let Ok(listener) = VsockListener::bind(&sockaddr) {
        println!("port1 bound listener");
        for conn in listener.incoming() {
            if let Ok(mut stream) = conn {
                println!("got conn on port1");

                let mut buffer = [0u8; BUFF_SIZE];

                loop {
                    dbg!(stream.write_all(&[3, 3, 3, 3, 3, 3]));

                    let nbytes = stream.read(&mut buffer);
                    let nbytes = match nbytes {
                        Err(_) => 0,
                        Ok(n) => n,
                    };

                    if nbytes > 0 {
                        println!("port1 read: {:02X?}", &buffer[..nbytes]);
                    } else {
                        std::thread::sleep(std::time::Duration::new(1, 0));
                    };
                }
            } else {
                println!("error getting conn on port1");
            }
        }
        Ok(())
    } else {
        Err(("port1 bound listener failed".to_owned()))
    }
}
