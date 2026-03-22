use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use xshell::{Shell, cmd};

use std::net::TcpStream;
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::{Duration, Instant};

#[derive(Parser)]
#[command(name = "xtask")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run example client against example server
    Example,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Example => example()?,
    }

    Ok(())
}

fn example() -> Result<()> {
    let sh = Shell::new()?;

    println!("Starting server...");

    let server = Command::new("cargo")
        .args(["run", "-p", "test-server"])
        .spawn()
        .context("failed to start server")?;

    let server = Arc::new(Mutex::new(server));

    setup_ctrlc(server.clone())?;

    wait_for_port("127.0.0.1:3000", Duration::from_secs(10))?;

    println!("Server ready");

    cmd!(sh, "cargo run -p single-http-request").run()?;

    println!("Stopping server...");
    kill_server(&server)?;

    Ok(())
}

fn wait_for_port(addr: &str, timeout: Duration) -> Result<()> {
    let start = Instant::now();

    while start.elapsed() < timeout {
        if TcpStream::connect(addr).is_ok() {
            return Ok(());
        }

        sleep(Duration::from_millis(100));
    }

    anyhow::bail!("server did not become ready at {addr}");
}

fn setup_ctrlc(server: Arc<Mutex<Child>>) -> Result<()> {
    ctrlc::set_handler(move || {
        println!("\nReceived Ctrl+C, shutting down server...");
        let _ = kill_server(&server);
        std::process::exit(1);
    })?;

    Ok(())
}

fn kill_server(server: &Arc<Mutex<Child>>) -> Result<()> {
    let mut child = server.lock().unwrap();
    let _ = child.kill();
    let _ = child.wait();
    Ok(())
}
