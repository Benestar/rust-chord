use ini::Ini;
use std::net::SocketAddr;
use std::path::Path;

#[derive(Debug)]
pub struct Config {
    pub listen_address: SocketAddr,
    pub api_address: SocketAddr,
}

impl Config {
    pub fn load_from_file<P: AsRef<Path>>(filename: P) -> ::Result<Config> {
        let conf = Ini::load_from_file(filename)?;

        let dht = conf.section(Some("dht"))
            .ok_or("missing section `dht`")?;

        let listen_address = dht.get("listen_address")
            .ok_or("missing value `listen_address`")?
            .parse()?;

        let api_address = dht.get("api_address")
            .ok_or("missing value `api_address`")?
            .parse()?;

        Ok(Config { listen_address, api_address })
    }
}
