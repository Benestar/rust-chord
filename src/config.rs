use ini::Ini;
use std::net::SocketAddr;
use std::path::Path;

#[derive(Copy, Clone, Debug)]
pub struct Config {
    pub listen_address: SocketAddr,
    pub api_address: SocketAddr,
    pub worker_threads: usize,
    pub timeout: u64,
    pub fingers: usize,
    pub stabilization_interval: u64,
}

impl Config {
    pub fn load_from_file<P: AsRef<Path>>(filename: P) -> crate::Result<Config> {
        let conf = Ini::load_from_file(filename)?;

        let dht = conf.section(Some("dht")).ok_or("missing section `dht`")?;

        let listen_address = dht
            .get("listen_address")
            .ok_or("missing value `listen_address`")?
            .parse()?;

        let api_address = dht
            .get("api_address")
            .ok_or("missing value `api_address`")?
            .parse()?;

        let worker_threads = dht
            .get("worker_threads")
            .unwrap_or(&"4".to_string())
            .parse()?;

        let timeout = dht
            .get("timeout")
            .unwrap_or(&"300000".to_string())
            .parse()?;

        let fingers = dht.get("fingers").unwrap_or(&"128".to_string()).parse()?;

        let stabilization_interval = dht
            .get("stabilization_interval")
            .unwrap_or(&"60".to_string())
            .parse()?;

        Ok(Config {
            listen_address,
            api_address,
            worker_threads,
            timeout,
            fingers,
            stabilization_interval,
        })
    }
}
