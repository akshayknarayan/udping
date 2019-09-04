use failure::Error;
use rand::seq::IteratorRandom;
use slog::Drain;
use std::net::ToSocketAddrs;
use std::sync::Mutex;
use structopt::StructOpt;

#[derive(Clone, StructOpt)]
struct Opt {
    #[structopt(short = "c")]
    server: String,

    #[structopt(short = "p")]
    port: u16,

    #[structopt(short = "n", default_value = "10")]
    num: usize,
}

fn main() -> Result<(), Error> {
    let opt = Opt::from_args();
    let decorator = slog_term::TermDecorator::new().build();
    let drain = Mutex::new(slog_term::FullFormat::new(decorator).build()).fuse();
    let log = slog::Logger::root(drain, slog::o!());

    let mut rng = rand::thread_rng();

    let sks: Vec<std::net::UdpSocket> = (5000..6000)
        .filter_map(|p| std::net::UdpSocket::bind(("0.0.0.0", p)).ok())
        .choose_multiple(&mut rng, opt.num);
    slog::debug!(log, "opened sks"; "num" => sks.len());

    let mut jhs = vec![];
    for sk in sks {
        let l = log.clone();
        let server = opt.clone().server;
        let port = opt.port;
        let jh = std::thread::spawn(move || {
            udping::ping(
                sk,
                (server.as_str(), port).to_socket_addrs()?.next().unwrap(),
                l,
            )
        });
        jhs.push(jh);
    }

    for jh in jhs {
        jh.join().unwrap()?;
    }

    Ok(())
}
