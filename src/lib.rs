use failure::Error;
use serde::{Deserialize, Serialize};
use slog::{info, trace};
use std::net::{ToSocketAddrs, UdpSocket};

#[derive(Deserialize, Serialize)]
struct TimeMsg(u64);

pub fn ping(
    sk: UdpSocket,
    peer: impl ToSocketAddrs + std::fmt::Debug + std::fmt::Display + Send + Clone + 'static,
    log: slog::Logger,
) -> Result<(), Error> {
    let client_sk = sk.try_clone()?;
    let client_peer = peer.clone();
    let client_log = log.clone();

    let client: std::thread::JoinHandle<Result<(), Error>> = std::thread::spawn(move || {
        let sk = client_sk;
        let peer = client_peer;
        info!(client_log, "client starting"; "peer" => ?peer);
        let mut recv_buf = [0u8; 1024];
        loop {
            let then = TimeMsg(time::precise_time_ns());
            let buf =
                serde_json::to_string(&then).map_err(|e| Error::from(e).context("serialize"))?;
            trace!(log, "sending");
            sk.send_to(&mut buf.as_bytes(), &peer)
                .map_err(|e| Error::from(e).context("send"))?;
            let (read, _) = sk.recv_from(&mut recv_buf)?;
            trace!(log, "received");
            let then: TimeMsg = serde_json::from_slice(&recv_buf[..read])
                .map_err(|e| Error::from(e).context("Deserialize"))?;
            let elapsed_ns = time::precise_time_ns() - then.0;
            info!(log, "Ping response"; "from" => format!("{}", peer), "local" => format!("{}", sk.local_addr().unwrap()),  "time" => elapsed_ns as f64 / 1e6);
            if elapsed_ns < 1_000_000 {
                let wait = 1_000_000 - elapsed_ns;
                std::thread::sleep(std::time::Duration::from_nanos(wait));
            }
        }
    });

    client
        .join()
        .unwrap()
        .map_err(|e| e.context("client error"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::TimeMsg;
    use failure::Error;

    #[test]
    fn serde() {
        let then = TimeMsg(time::precise_time_ns());
        let buf = serde_json::to_string(&then)
            .map_err(|e| Error::from(e).context("serialize"))
            .unwrap();
        let recv_buf = buf.as_bytes();
        let now: TimeMsg = serde_json::from_slice(&recv_buf)
            .map_err(|e| Error::from(e).context("Deserialize"))
            .unwrap();

        assert_eq!(then.0, now.0);
    }
}
