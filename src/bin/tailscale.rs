use std::collections::HashMap;
use std::io::Read;
use std::net::{IpAddr, SocketAddrV6};
use std::process::{Command, Stdio};

use clap::Parser;
use serde::Deserialize;

fn run_command(cmd: &str, args: Vec<&str>) -> Result<String, String> {
    let mut cmd = Command::new(cmd);
    cmd.args(args);
    cmd.stderr(Stdio::null());
    cmd.stdout(Stdio::piped());
    let mut handle = cmd.spawn().map_err(|err| err.to_string())?;
    let Some(mut stdout) = handle.stdout.take() else {
        return Err(String::from("Failed to get stdout from child"));
    };
    let mut buf = vec![];
    stdout.read_to_end(&mut buf).expect("Failed to read");
    String::from_utf8(buf).map_err(|err| err.to_string())
}

#[derive(Debug, Deserialize)]
struct TailscaleStatus {
    #[serde(rename = "Peer")]
    peers: HashMap<String, TailscalePeer>,
}

#[derive(Debug, Deserialize)]
struct TailscalePeer {
    #[serde(rename = "TailscaleIPs")]
    ips: Vec<IpAddr>,
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, default_value = "[ff02::1213:1989]:7475")]
    multicast_address: SocketAddrV6,
}

fn main() {
    let args = Args::parse();
    let ip = {
        let output = run_command("tailscale", vec!["ip", "--6"])
            .expect("Failed to get local Tailscale IP address");
        output.trim().to_owned()
    };
    let status: TailscaleStatus = {
        let json = run_command(
            "tailscale",
            vec!["status", "--json", "--self=false"], // todo: --active?
        )
        .expect("Failed to get status");
        serde_json::from_str(&json).expect("Failed to parse JSON")
    };
    let target_ips = status
        .peers
        .values()
        .filter_map(|peer| {
            peer.ips
                .iter()
                .filter_map(|ip| {
                    if ip.is_ipv6() {
                        Some(ip.to_string())
                    } else {
                        None
                    }
                })
                .collect::<Vec<String>>()
                .first()
                .cloned()
        })
        .collect::<Vec<_>>();

    let mut argv: Vec<String> = vec![
        "run",
        "--bin",
        "crosswind",
        "--",
        "--interface",
        &ip,
        "--multicast-address",
        &args.multicast_address.to_string(),
    ]
    .iter()
    .map(|str| str.to_string())
    .collect();

    for ip in target_ips.iter() {
        argv.push(String::from("--targets"));
        argv.push(format!("[{ip}]:9908"));
    }

    let err = exec::Command::new("/home/ubuntu/.cargo/bin/cargo").args(&argv).exec();
    panic!("Error: {}", err);
}
