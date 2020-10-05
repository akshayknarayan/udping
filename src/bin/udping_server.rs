use color_eyre::eyre::Error;
use slog::Drain;
use std::sync::Mutex;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Opt {
    #[structopt(short = "p")]
    port: u16,
}

fn main() -> Result<(), Error> {
    color_eyre::install()?;
    let opt = Opt::from_args();
    let decorator = slog_term::TermDecorator::new().build();
    let drain = Mutex::new(slog_term::FullFormat::new(decorator).build()).fuse();
    let log = slog::Logger::root(drain, slog::o!());

    let sk = std::net::UdpSocket::bind(("0.0.0.0", opt.port))?;
    let mut recv_buf = [0u8; 1024];
    loop {
        let (read, addr) = sk.recv_from(&mut recv_buf)?;
        slog::trace!(log, "sending");
        sk.send_to(&mut recv_buf[..read], &addr)?;
    }
}
