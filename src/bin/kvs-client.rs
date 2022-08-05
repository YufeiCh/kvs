use std::net::SocketAddr;
use std::process::exit;

use structopt::{clap::AppSettings, StructOpt};

use kvs::{KvsClient, Result};

const DEFAULT_LISTENNING_ADDRESS: &str = "127.0.0.1:4000";

#[derive(Debug, StructOpt)]
#[structopt(
    name = "kvs-client",
    global_settings = &[
        AppSettings::DisableHelpSubcommand,
        AppSettings::VersionlessSubcommands
    ],
)]
struct Opt {
    #[structopt(subcommand)]
    command: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    #[structopt(name = "get", about = "get the string value of a given key")]
    Get {
        #[structopt(name = "KEY", help = "a string key")]
        key: String,
        #[structopt(value_name = "IP:PORT", long, help = "the server address", default_value = DEFAULT_LISTENNING_ADDRESS, parse(try_from_str))]
        addr: SocketAddr,
    },
    #[structopt(name = "set", about = "set the key valur string to the store")]
    Set {
        #[structopt(name = "KEY", help = "a string key")]
        key: String,
        #[structopt(name = "VALUE", help = "a string value")]
        value: String,
        #[structopt(value_name = "IP:PORT", long, help = "the server address", default_value = DEFAULT_LISTENNING_ADDRESS, parse(try_from_str))]
        addr: SocketAddr,
    },
    #[structopt(name = "rm", about = "remove the string value of a given key")]
    Remove {
        #[structopt(name = "KEY", help = "a string key")]
        key: String,
        #[structopt(value_name = "IP:PORT", long, help = "the server address", default_value = DEFAULT_LISTENNING_ADDRESS, parse(try_from_str))]
        addr: SocketAddr,
    },
}

fn main() {
    let opt = Opt::from_args();
    if let Err(err) = run(opt) {
        eprintln!("{}", err);
        exit(1);
    }
}

fn run(opt: Opt) -> Result<()> {
    match opt.command {
        Command::Get { key, addr } => {
            let mut client = KvsClient::connect(addr)?;
            if let Some(value) = client.get(key)? {
                println!("{}", value);
            } else {
                println!("Key not found");
            }
        }
        Command::Set { key, value, addr } => {
            let mut client = KvsClient::connect(addr)?;
            client.set(key, value)?;
        }
        Command::Remove { key, addr } => {
            let mut client = KvsClient::connect(addr)?;
            client.remove(key)?;
        }
    }
    Ok(())
}
