use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

use clap::Parser;
use uuid::Uuid;

use crate::state::{storage::StateFile, SavedContainer};

mod state;

struct PortDefinition {
    host: u16,
    container: u16,
}

struct EnvironmentVariable {
    name: String,
    value: String,
}

struct VolumeDefinition {
    local_path: PathBuf,
    container_path: String,
}

struct ContainerDefinition {
    name: String,
    ports: Vec<PortDefinition>,
    environment: Vec<EnvironmentVariable>,
    volumes: Vec<VolumeDefinition>,
}

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

    let state_file_path = state_path.clone().join("containers.json");
    let mut state_file = StateFile::new(state_file_path);
    let containers = vec![ContainerDefinition {
        name: "postgres".to_string(),
        ports: vec![PortDefinition {
            host: 5432,
            container: 5432,
        }],
        environment: vec![
            EnvironmentVariable {
                name: "POSTGRES_PASSWORD".to_string(),
                value: "postgres".to_string(),
            },
            EnvironmentVariable {
                name: "POSTGRES_DB".to_string(),
                value: "app".to_string(),
            },
        ],
        volumes: vec![VolumeDefinition {
            local_path: project_root.join("docker/pginit"),
            container_path: "/docker-entrypoint-initdb.d/".to_string(),
        }],
    }];

    for container in &containers {
        if let Some(saved_container) = state_file.state().container(&container.name) {
            Command::new("docker")
                .args(["start", saved_container.docker_id()])
                .spawn()
                .unwrap();
        } else {
            let mut args = vec!["run".to_string()];

            for volume in &container.volumes {
                let local_path = volume
                    .local_path
                    .to_str()
                    .expect("Local volume path is not valid Unicode");
                args.push("-v".to_string());
                args.push(format!("{}:{}:ro", local_path, volume.container_path));
            }

            for port in &container.ports {
                args.push("-p".to_string());
                args.push(format!("{}:{}", port.container, port.host));
            }

            for environment_variable in &container.environment {
                args.push("-e".to_string());
                args.push(format!(
                    "{}={}",
                    environment_variable.name, environment_variable.value
                ));
            }

            args.push("--name".to_string());
            args.push(format!("{}-{}", container.name, Uuid::new_v4()));

            args.push("-d".to_string());

            let cmd = Command::new("docker")
                .args(args)
                .stdout(Stdio::piped())
                .spawn()
                .expect("Failed to execute docker run");

            let output = cmd
                .wait_with_output()
                .expect("Failed to wait for docker run");
            let stdout =
                String::from_utf8(output.stdout).expect("docker run output was not valid unicode");

            state_file
                .state_mut()
                .add_container(
                    container.name.clone(),
                    SavedContainer::from_docker_id(stdout.trim().to_string()),
                )
                .expect("Failed to save container state");
        }
    }

    Command::new("cargo")
        .args(["sqlx", "migrate", "run"])
        .current_dir(project_root.join("apps/backend/"))
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}
