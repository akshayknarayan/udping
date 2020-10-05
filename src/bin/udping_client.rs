use color_eyre::eyre::{Error, WrapErr};
use slog::Drain;
use std::net::ToSocketAddrs;
use std::sync::Mutex;
use structopt::StructOpt;
use tokio::net::UdpSocket;

#[derive(Clone, StructOpt)]
struct Opt {
    #[structopt(short = "c")]
    server: String,

    #[structopt(short = "p")]
    port: u16,

    #[structopt(short = "n", default_value = "10")]
    num: usize,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    color_eyre::install()?;
    let opt = Opt::from_args();
    let decorator = slog_term::TermDecorator::new().build();
    let drain = Mutex::new(slog_term::FullFormat::new(decorator).build()).fuse();
    let log = slog::Logger::root(drain, slog::o!());

    //let mut rng = rand::thread_rng();
    //let sks: Vec<std::net::UdpSocket> = (5000..6000)
    //    .filter_map(|p| std::net::UdpSocket::bind(("0.0.0.0", p)).ok())
    //    .choose_multiple(&mut rng, opt.num);
    let mut sks = vec![];
    for p in 5980..5990 {
        let ip: std::net::IpAddr = "0.0.0.0".parse()?;
        let sk = UdpSocket::bind((ip, p as u16)).await.wrap_err("bind")?;
        sks.push(sk);
    }

    slog::debug!(log, "opened sks"; "num" => sks.len());

    let mut futs = vec![];
    for sk in sks {
        let l = log.clone();
        let server = opt.clone().server;
        let port = opt.port;
        let f = udping::ping(sk, (server, port).to_socket_addrs()?.next().unwrap(), l);

        futs.push(f);
    }

    futures_util::future::join_all(futs).await;

    Ok(())
}
