use std::io::Read;
use std::process::{Command, Stdio};

use serde_json::Value;

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

fn main() {
    let ip = {
        let output = run_command("tailscale", vec!["ip", "--6"])
            .expect("Failed to get local Tailscale IP address");
        output.trim().to_owned()
    };
    let status: Value = {
        let json = run_command(
            "tailscale",
            vec!["status", "--json", "--self=false"], // todo: --active?
        )
        .expect("Failed to get status");
        serde_json::from_str(&json).expect("Failed to parse JSON")
    };
    let target_ips = match status.get("Peer") {
        Some(Value::Object(peer_map)) => peer_map
            .values()
            .map(|peer_info| {
                // "May our children forgive us."
                // -- President Whitmore
                peer_info
                    .get("TailscaleIPs")
                    .unwrap()
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|ip| ip.as_str().unwrap())
                    .find(|ip| ip.contains(":"))
                    .expect("Peer did not have an IPv6 address")
                    .to_string()
            })
            .collect::<Vec<String>>(),
        _ => panic!("Failed to get list of peers: {:?}", status.get("Peer")),
    };

    let mut argv: Vec<String> = vec![
        "run",
        "--bin",
        "crosswind",
        "--",
        "--interface",
        &ip,
        "--multicast-address",
        "[ff02::1213:1989]:7475",
    ]
    .iter()
    .map(|str| str.to_string())
    .collect();

    for ip in target_ips.iter() {
        argv.push(String::from("--targets"));
        argv.push(format!("[{ip}]:9908"));
    }

    println!("{argv:?}");
    let err = exec::Command::new("cargo").args(&argv).exec();
    panic!("Error: {}", err);
}
