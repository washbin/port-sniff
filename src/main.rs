use std::{
    env,
    net::{IpAddr, SocketAddr, TcpStream},
    str::FromStr,
    sync::mpsc::{channel, Sender},
    thread,
    time::Duration,
};

const MAX_PORT: u16 = u16::MAX;

struct Arguments {
    ipaddr: IpAddr,
    threads: u16,
}

impl Arguments {
    fn new(args: &[String]) -> Result<Arguments, &'static str> {
        match args.len() {
            1 | 3 => Err("Not enough arguments"),
            2 => match IpAddr::from_str(&args[1]) {
                Ok(ipaddr) => Ok(Arguments {
                    ipaddr,
                    threads: 400,
                }),
                _ => Err("help"),
            },
            4 => {
                if &args[1] != "-j" {
                    return Err("help");
                }
                match args[2].parse::<u16>() {
                    Ok(threads) => match IpAddr::from_str(&args[3]) {
                        Ok(ipaddr) => Ok(Arguments { ipaddr, threads }),
                        _ => Err("help"),
                    },
                    Err(_) => Err("help"),
                }
            }
            _ => Err("Too many arguments"),
        }
    }
}

fn main() {
    let args = match Arguments::new(&env::args().collect::<Vec<String>>()) {
        Ok(args) => args,
        Err(err) => {
            if err != "help" {
                eprintln!("{}", err)
            }
            eprintln!("Usage: port-sniff -j [Number of threads] [ip_address]");
            return;
        }
    };
    let num_threads = args.threads;

    let (tx, rx) = channel();
    for port in 0..num_threads {
        let tx = tx.clone();
        thread::spawn(move || scan(tx, port, args.ipaddr, num_threads.into()));
    }

    for v in rx {
        println!("{} is open", v);
    }
}

fn scan(tx: Sender<u16>, start_port: u16, ip_address: IpAddr, num_threads: usize) {
    for port in (start_port..MAX_PORT).step_by(num_threads) {
        if TcpStream::connect_timeout(
            &SocketAddr::new(ip_address, port),
            Duration::from_millis(300),
        )
        .is_ok()
        {
            tx.send(port)
                .expect("Failed to send port to consumer channel");
        }
    }
}
