use std::{
    io::Read,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    project_root: PathBuf,
}

fn main() {
    let cli = Cli::parse();
    let project_root = std::fs::canonicalize(cli.project_root).unwrap();

    println!("Running in: {}", project_root.to_string_lossy());

    let state_path = project_root.join(".state");
    std::fs::create_dir_all(state_path.clone()).unwrap();

    let mut containers = std::fs::File::options()
        .write(true)
        .read(true)
        .create(true)
        .open(state_path.clone().join("containers"))
        .unwrap();

    let mut current_container_id = String::new();
    containers
        .read_to_string(&mut current_container_id)
        .unwrap();

    if current_container_id.is_empty() {
        println!("No container found. Creating a new one...");

        let cmd = Command::new("docker")
            .args([
                "run",
                "-p",
                "5432:5432",
                "--name",
                "backend-pgsql",
                "-e",
                "POSTGRES_PASSWORD=postgres",
                "-e",
                "POSTGRES_DB=app",
                "-d",
                "postgres",
            ])
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let o = cmd.wait_with_output().unwrap();
        let stdout = String::from_utf8_lossy(&o.stdout);

        current_container_id = stdout.trim().to_string();

        write!(containers, "{}", current_container_id).unwrap();
        containers.flush().unwrap();

        println!("Container created");
    } else {
        println!("Container already exists with ID: {}", current_container_id);

        Command::new("docker")
            .args(["start", &current_container_id])
            .spawn()
            .unwrap();

        println!("Container started");
    }

    Command::new("cargo")
        .args(["sqlx", "migrate", "run"])
        .current_dir(project_root.join("apps/backend/"))
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}
