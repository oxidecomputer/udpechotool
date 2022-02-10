// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::net::{ToSocketAddrs, UdpSocket};
use std::time::{Duration, Instant};
use structopt::StructOpt;

#[derive(StructOpt)]
struct UdpEcho {
    #[structopt(long)]
    source: Option<String>,

    target: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = UdpEcho::from_args();

    let dest = args.target.to_socket_addrs()?.collect::<Vec<_>>();

    let socket = if let Some(src) = args.source {
        UdpSocket::bind(src)?
    } else {
        UdpSocket::bind("[::]:0")?
    };
    socket.set_read_timeout(Some(Duration::from_millis(250)))?;
    socket.connect(&dest[..])?;

    let peer = socket.peer_addr()?;

    let packet = [0xAA; 64];

    let mut buf = [0; 64];

    println!("UDPECHO {} from {}: {} data bytes",
        peer, socket.local_addr()?, packet.len());
    loop {
        let send_time = Instant::now();
        socket.send(&packet)?;

        let timeout = Instant::now() + Duration::from_secs(1);

        loop {
            match socket.recv(&mut buf) {
                Ok(n) => {
                    let recv_time = Instant::now();
                    println!("{} bytes from {}: time = {:?}",
                        n, peer, recv_time - send_time);
                    break;
                }
                Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {
                    if timeout <= Instant::now() {
                        break;
                    }
                }
                Err(e) => {
                    return Err(e.into());
                }
            }
        }

        if let Some(left) = timeout.checked_duration_since(Instant::now()) {
            std::thread::sleep(left);
        }
    }
}
