use clap::{builder::Command, arg};

fn main() {
    let matches = Command::new(env!("CARGO_PKG_NAME"))
    .version(env!("CARGO_PKG_VERSION"))
    .author(env!("CARGO_PKG_AUTHORS"))
    .subcommand_required(true)
    .subcommand(
        Command::new("set")
        .about("set the key value to KvStore")
        .arg(arg!([key]))
        .arg(arg!([value]))
    )
    .subcommand(
        Command::new("get")
        .about("get the value from KvStore by the key")
        .arg(arg!([key]))
    )
    .subcommand(
        Command::new("rm")
        .about("remove the key/value from KvStore by the key")
        .arg(arg!([key]))
    )
    .get_matches();

    match matches.subcommand() {
        Some(("set", _)) => {
            unimplemented!("unimplemented");
        },
        Some(("get",_)) => {
            unimplemented!("unimplemented");
        },
        Some(("rm",_)) => {
            unimplemented!("unimplemented");
        },
        _ => unreachable!("unimplemented"),
    }
}
