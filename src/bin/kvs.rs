use clap::{Command, Arg};
use kvs::{Result, KvStore, KvsError};
use std::env::current_dir;
use std::process::exit;

fn main() -> Result<()> {
    let cmd = Command::new(env!("CARGO_PKG_NAME"))
    .version(env!("CARGO_PKG_VERSION"))
    .author(env!("CARGO_PKG_AUTHORS"))
    .subcommand_required(true)
    .subcommand(
        Command::new("set")
        .about("set the key value to KvStore")
        .arg(Arg::with_name("KEY").help("A String key").required(true))
        .arg(Arg::with_name("VALUE").help("A String value").required(true))
    )
    .subcommand(
        Command::new("get")
        .about("get the value from KvStore by the key")
        .arg(Arg::with_name("KEY").help("A String key").required(true))
    )
    .subcommand(
        Command::new("rm")
        .arg(Arg::with_name("KEY").help("A String key").required(true))
    );

    let matches = cmd.get_matches();

    match matches.subcommand() {
        Some(("set", matches)) => {
            let key = matches.get_one::<String>("KEY").unwrap();
            let value = matches.get_one::<String>("VALUE").unwrap();

            let mut store = KvStore::open(current_dir()?)?;
            store.set(key.to_string(), value.to_string())?;
        }
        Some(("get", matches)) => {
            let key = matches.get_one::<String>("KEY").unwrap();

            let mut store = KvStore::open(current_dir()?)?;
            if let Some(value) = store.get(key.to_string())? {
                println!("{}", value);
            } else {
                println!("Key not found");
            }
        }
        Some(("rm", matches)) => {
            let key = matches.get_one::<String>("KEY").unwrap();

            let mut store = KvStore::open(current_dir()?)?;
            match store.remove(key.to_string()) {
                Ok(()) => {},
                Err(KvsError::KeyNotFound) => {
                    println!("Key not found");
                    exit(1);
                }
                Err(e) => return Err(e),
            } ;
        }
        _ => unreachable!("unimplemented"),
    }
    Ok(())
}