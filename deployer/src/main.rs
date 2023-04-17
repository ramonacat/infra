use std::process::Command;

enum Proxy {
    Ip(String),
    None,
}

struct FlakeTarget(String);

fn build_nixos_rebuild_command(
    target_host: &str,
    proxy: Proxy,
    flake_target: FlakeTarget,
) -> Command {
    let mut command = Command::new("nixos-rebuild");
    command
        .arg("--target-host")
        .arg(format!("root@{}", target_host))
        .arg("--build-host")
        .arg(format!("root@{}", target_host))
        .arg("--flake")
        .arg(format!(".#{}", flake_target.0))
        .arg("--fast") // workaround for https://github.com/NixOS/nixpkgs/issues/177873
        .arg("switch");

    if let Proxy::Ip(ip) = proxy {
        command.env("NIX_SSHOPTS", format!("-J root@{}", ip)); // TODO: support jumping as other users?
    }

    command
}

fn main() {
    let output = Command::new("terraform")
        .arg("output")
        .arg("-json")
        .output()
        .expect("Failed to execute `terraform output -json`");

    if !output.status.success() {
        panic!("terraform output -json returned an error");
    }

    let value = serde_json::from_slice::<serde_json::Value>(&output.stdout)
        .expect("Failed to deserialize the json output");

    let ips = value
        .get("node_ips")
        .and_then(|v| v.get("value"))
        .and_then(|v| v.as_array())
        .and_then(|v| {
            Some(
                v.iter()
                    .map(|x| x.as_str().expect("Failed to retreive the IP value"))
                    .collect::<Vec<_>>(),
            )
        });

    let Some(_ips) = ips else {
        panic!("Failed to receive node IPs");
    };

    let mut command = build_nixos_rebuild_command("jump.ramona.fun", Proxy::None, FlakeTarget("jump".into()));
    let output = command.output().expect("Failed to start nixos-rebuild");

    println!("{}", String::from_utf8_lossy(&output.stderr));
    println!("{}", String::from_utf8_lossy(&output.stdout));
}
