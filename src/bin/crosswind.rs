use clap::Parser;
use crosswind::{create_broadcast_sockets, get_interface};
use std::{
    net::{IpAddr, SocketAddr, SocketAddrV6},
    process::exit,
    time::Duration,
};
use tokio::{net::UdpSocket, time::sleep};

#[derive(Parser, Debug)]
struct Args {
    #[arg(long)]
    interface: String,
    #[arg(long)]
    multicast_address: SocketAddrV6,
    #[arg(long, required = true)]
    targets: Vec<SocketAddrV6>,
    #[arg(long, default_value = "9908")]
    port: u16,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let Some(interface) = get_interface(&args.interface) else {
        eprintln!("No available interfaces match '{}'", args.interface);
        exit(1);
    };

    println!(
        "Forwarding multicast traffic from multicast address:\n\t{}\narriving via interface:\n\t{} ({})\nto the following targets:\n\t{}",
        args.multicast_address,
        interface.name,
        interface.ip(),
        args.targets
            .iter()
            .map(|addr| addr.to_string())
            .collect::<Vec<_>>()
            .join("\n\t")
    );
    let listen_addr = {
        let ip = match interface.ip() {
            IpAddr::V4(_) => panic!("This should never happen"),
            IpAddr::V6(ip) => ip,
        };
        SocketAddr::new(IpAddr::V6(ip), args.port)
    };
    println!(
        "and forwarding traffic received at:\n\t{}\nto multicast address:\n\t{}",
        listen_addr, args.multicast_address
    );

    let (broadcast_in_sock, broadcast_out_sock) =
        create_broadcast_sockets(&interface, &args.multicast_address)
            .expect("Failed to create broadcast sockets");
    let direct_sock = UdpSocket::bind(format!("[::]:{}", args.port))
        .await
        .expect("Failed to create direct socket");

    const INCOMING_BUFFER_SIZE: usize = 1024;
    let mut buf = [0; INCOMING_BUFFER_SIZE];
    loop {
        // Handle incoming multicast message and forward them to our targets
        while let Ok((len, sender)) = broadcast_in_sock.try_recv_from(&mut buf) {
            if sender.port() == broadcast_out_sock.local_addr().unwrap().port() {
                // Ignore stuff from ourselves
                // TODO: check IP address too, not just port -- this is weirdly more complicated
                // than it sounds, because some interfaces have multiple inet6 addresses.
                continue;
            }

            println!("[M:IN] {len} bytes from {sender}");
            for target in args.targets.iter() {
                println!("[D:OUT] {len} bytes to {target}");
                direct_sock
                    .send_to(&buf[0..len], target)
                    .await
                    .expect("Failed to forward data");
            }
        }

        // Handle incoming messages directed at us and re-broadcast them over multicast
        while let Ok((len, sender)) = direct_sock.try_recv_from(&mut buf) {
            println!("[D:IN] {len} bytes from {sender}");
            broadcast_out_sock
                .send_to(&buf[0..len], args.multicast_address)
                .await
                .expect("Failed to broadcast data");
            println!("[M:OUT] {len} bytes to {}", args.multicast_address);
        }

        sleep(Duration::from_millis(500)).await;
    }
}
