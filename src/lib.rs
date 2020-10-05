use color_eyre::eyre::{Error, WrapErr};
use serde::{Deserialize, Serialize};
use slog::{info, trace};
use tokio::net::{ToSocketAddrs, UdpSocket};

#[derive(Deserialize, Serialize)]
struct TimeMsg(u64);

pub async fn ping(
    mut sk: UdpSocket,
    peer: impl ToSocketAddrs + std::fmt::Debug + std::fmt::Display + Send + Clone + 'static,
    log: slog::Logger,
) -> Result<(), Error> {
    info!(&log, "client starting"; "peer" => ?peer);
    let mut recv_buf = [0u8; 1024];
    loop {
        let then = TimeMsg(time::precise_time_ns());
        let buf = serde_json::to_string(&then).wrap_err("serialize")?;
        trace!(&log, "sending");
        sk.send_to(&mut buf.as_bytes(), &peer)
            .await
            .wrap_err("send")?;
        let (read, _) = sk.recv_from(&mut recv_buf).await.wrap_err("recv")?;
        trace!(&log, "received");
        let then: TimeMsg = serde_json::from_slice(&recv_buf[..read]).wrap_err("Deserialize")?;
        let elapsed_ns = time::precise_time_ns() - then.0;
        info!(&log, "Ping response"; "from" => format!("{}", peer), "local" => format!("{}", sk.local_addr().unwrap()),  "time" => elapsed_ns as f64 / 1e6);
        if elapsed_ns < 1_000_000 {
            let wait = 1_000_000 - elapsed_ns;
            tokio::time::delay_for(tokio::time::Duration::from_nanos(wait)).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TimeMsg;
    use color_eyre::eyre::WrapErr;

    #[test]
    fn serde() {
        let then = TimeMsg(time::precise_time_ns());
        let buf = serde_json::to_string(&then).wrap_err("serialize").unwrap();
        let recv_buf = buf.as_bytes();
        let now: TimeMsg = serde_json::from_slice(&recv_buf)
            .wrap_err("Deserialize")
            .unwrap();

        assert_eq!(then.0, now.0);
    }
}
