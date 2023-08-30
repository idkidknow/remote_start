use api;
use colored::Colorize;
use common::error::Error;
use dotenvy;
use tokio::{sync::{Notify, watch}, process::Command};
use std::{env, net::SocketAddr, str::FromStr, sync::Arc, process::ExitStatus};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let _ = dotenvy::dotenv(); // Read .env
    let socket_addr = read_env("ADDR", Some(SocketAddr::from_str("0.0.0.0:25566").unwrap()))?;
    let command: String = read_env("COMMAND", None)?;
    let (tx, rx) = watch::channel(false); // watch if the subprocess is started
    let notify = Arc::new(Notify::new());
    let notify2 = notify.clone();
    let api_task = tokio::spawn(async move {
        println!("Listening {}", socket_addr.to_string().cyan());
        api::start(socket_addr, notify, rx).await
    });
    let command_task = tokio::spawn(async move {
        subprocess_loop(&command, notify2, tx).await
    });
    let results = tokio::join!(api_task, command_task);
    if results.0.is_err() || results.1.is_err() {
        return Err(Error::Error("Fatal error"));
    }
    Ok(())
}

fn read_env<T>(name: &'static str, default_value: Option<T>) -> Result<T, Error>
where
    T: FromStr,
{
    let env_str = match env::var(name) {
        Ok(e) => e,
        Err(_) => {
            return match default_value {
                None => Err(Error::EnvError {
                    variable_name: name,
                }),
                Some(d) => Ok(d),
            };
        }
    };

    let value = env_str.parse::<T>();
    match value {
        Ok(v) => Ok(v),
        Err(_) => {
            return Err(Error::EnvError {
                variable_name: name,
            });
        }
    }
}

async fn subprocess_loop(command: &str, notify: Arc<Notify>, started_sender: watch::Sender<bool>) {
    loop {
        started_sender.send(true).expect("API server down");
        let res = run_command(command).await;
        started_sender.send(false).expect("API server down");
        match res {
            Err(e) => {
                eprintln!("Error when executing command: {}", e);
            },
            Ok(status) => {
                println!("{}", status.to_string().red());
            },
        }
        println!("{}", "Waiting for request".cyan());
        notify.notified().await;
    }
}

#[cfg(target_os = "linux")]
async fn run_command(command: &str) -> Result<ExitStatus, Error> {
    let status = Command::new("sh")
        .args(["-c", command])
        .spawn()?
        .wait()
        .await?;
    Ok(status)
}

#[cfg(target_os = "windows")]
async fn run_command(command: &str) -> Result<ExitStatus, Error> {
    let status = Command::new("cmd")
        .args(["/C", command])
        .spawn()?
        .wait()
        .await?;
    Ok(status)
}
