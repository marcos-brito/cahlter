use anyhow::Result;
use cahlter::vault::Vault;
use clap::{Arg, ArgMatches, Command};
use colored::Colorize;
use human_panic::setup_panic;
use is_terminal::IsTerminal;
use log::{error, info, kv, warn, Level};
use std::env;
use std::io::Write;

const DEFAULT_PORT: &str = "8080";

fn setup_logging() {
    if !std::io::stdout().is_terminal() {
        env_logger::init();
        return;
    }

    let env = env_logger::Env::default().default_filter_or("info");
    env_logger::Builder::from_env(env)
        .format(|buf, record| match record.level() {
            Level::Error => writeln!(buf, "❌ {}: {}", "error".red().bold(), record.args(),),
            Level::Warn => writeln!(buf, "⚠️ {}: {}", "warn".yellow().bold(), record.args()),
            Level::Info => writeln!(
                buf,
                "{} {}",
                record
                    .key_values()
                    .get(kv::Key::from("emoji"))
                    .unwrap_or(kv::Value::from(""))
                    .to_string(),
                record.args()
            ),
            _ => writeln!(buf, "{} {}", record.level(), record.args()),
        })
        .init();
}

fn cli() -> Command {
    Command::new("calhter")
        .about("A minimalistic static web site generator")
        .subcommand_required(true)
        .subcommand(Command::new("init").arg(Arg::new("vault_path").help("The vault's path")))
        .subcommand(Command::new("build").arg(Arg::new("vault_path").help("The vault's path")))
        .subcommand(
            Command::new("serve")
                .arg(Arg::new("port").long("port"))
                .arg(Arg::new("vault_path").help("The vault's path")),
        )
}

#[async_std::main]
async fn main() -> Result<()> {
    setup_logging();
    setup_panic!();

    if let Err(e) = run().await {
        error!("{}", e);
        std::process::exit(1);
    }

    Ok(())
}

async fn run() -> Result<()> {
    let matches = cli().get_matches();
    match matches.subcommand() {
        Some(("init", submatches)) => init(submatches)?,
        Some(("build", submatches)) => build(submatches)?,
        Some(("serve", submatches)) => serve(submatches).await?,
        _ => unreachable!(),
    };

    Ok(())
}

fn init(matches: &ArgMatches) -> Result<()> {
    info!(emoji= "⚙️"; "Preparing the vault...");
    let vault_path = matches
        .get_one::<String>("vault_path")
        .and_then(|s| Some(s.as_str()))
        .unwrap_or(".");

    let mut vault = match vault_path.starts_with("/") {
        true => Vault::new(vault_path),
        false => {
            let current_dir = env::current_dir().expect("Could not get the current dir");
            Vault::new(current_dir.join(vault_path))
        }
    };

    info!(
        emoji="⚡"; "Initializing the vault at {}...",
        vault.path.display()
    );

    vault.init()?;

    info!(emoji = "✅"; "Done");

    Ok(())
}

fn build(matches: &ArgMatches) -> Result<()> {
    info!(emoji = "💿"; "Reading the vault...");
    let vault_path = matches
        .get_one::<String>("vault_path")
        .and_then(|s| Some(s.as_str()))
        .unwrap_or(".");

    let mut vault = match vault_path.starts_with("/") {
        true => Vault::from_disk(vault_path)?,
        false => {
            let current_dir = env::current_dir().expect("Could not get the current dir");
            Vault::from_disk(current_dir.join(vault_path))?
        }
    };

    info!(emoji = "🏗️"; "Building...");
    vault.build()?;

    info!(emoji = "✅"; "Done");
    Ok(())
}

async fn serve(matches: &ArgMatches) -> Result<()> {
    info!(emoji = "💿"; "Reading the vault...");
    let vault_path = matches
        .get_one::<String>("vault_path")
        .and_then(|s| Some(s.as_str()))
        .unwrap_or(".");

    let vault = match vault_path.starts_with("/") {
        true => Vault::from_disk(vault_path)?,
        false => {
            let current_dir = env::current_dir().expect("Could not get the current dir");
            Vault::from_disk(current_dir.join(vault_path))?
        }
    };

    let mut app = tide::new();
    let port = match matches.get_one::<String>("port") {
        Some(p) => p,
        None => {
            warn!("No port specified. Using default: {}", DEFAULT_PORT);
            DEFAULT_PORT
        }
    };

    info!(emoji = "🌐"; "Starting the server");
    app.at("/").serve_dir(vault.build_dir())?;
    app.listen("127.0.0.1:".to_string() + &port).await?;
    Ok(())
}
