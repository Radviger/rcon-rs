extern crate rcon;

use std::io::stdin;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::time::Duration;
use clap::Parser;
use rcon::RCon;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    ip: Ipv4Addr,
    #[arg(short, long)]
    port: u16,
    #[arg(long)]
    password: String,
    #[arg(long, value_parser = humantime::parse_duration, default_value = "1s")]
    connect_timeout: Duration,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let addr = SocketAddr::V4(SocketAddrV4::new(args.ip, args.port));
    let mut con = RCon::connect(&addr, args.connect_timeout)?;
    let authorized = con.authorize(&args.password)?;

    if !authorized {
        eprint!("Unauthorized");
    } else {
        eprintln!("Connected");
    }

    let mut command = String::new();
    loop {
        let len = stdin().read_line(&mut command)?;
        if len == 0 {
            break;
        }
        let result = con.command(command.trim_end())?;
        println!("{}", result);
    }

    Ok(())
}