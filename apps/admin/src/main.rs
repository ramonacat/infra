use std::path::PathBuf;
use clap::{Parser, Subcommand};
use reqwest::Url;
use serde::Serialize;
use uuid::Uuid;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    token: String,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Post {
        title: String,
        content: PathBuf
    }
}

#[derive(Serialize)]
struct CreatePostRequest {
    id: Uuid,
    title: String,
    content: String,
}

// JRmKu2c6Kfl8ms5mlhMCfkNyzxGrprTRZmpQvPIB0iQcjzVqUgpCjJIGpClqOxEYR7mguSG5JaOuXPxPJe6UWvQSyZK9BT8zdahRuHLKeiiwDsfZ0NcKOVZAz1Ssnqwp
#[tokio::main]
async fn main() {
    let base = Url::parse("http://localhost:8080/").unwrap();
    let reqwest = reqwest::Client::new();
    let cli = Cli::parse();

    match cli.command {
        Commands::Post { title, content } => {
           println!("Publishing: {} from {:?}", title, content); 

           let content = std::fs::read_to_string(content).unwrap();

           let response = reqwest.post(base.join("api/posts").unwrap())
                .json(&CreatePostRequest { id: Uuid::new_v4(), title, content })
                .header("X-Token", cli.token)
                .send()
                .await
                .unwrap();

            println!("{:?}", response);
        },
    }
    println!("Hello, world!");
}
