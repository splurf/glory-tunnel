mod config;
mod tools;

use {
    config::Config,
    std::io::{Error, Result},
    tools::Tunnel,
};

fn main() -> Result<()> {
    let config = Config::from_args().map_err(Error::from)?;
    let tunnel = Tunnel::try_from(config)?;
    tunnel.init()
}
