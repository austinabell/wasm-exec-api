#![recursion_limit = "1024"]

mod config;
mod local_db;
mod logger;
mod server;
mod utils;

#[cfg(feature = "p2p")]
mod p2p;

#[cfg(not(feature = "p2p"))]
#[async_std::main]
async fn main() -> tide::Result<()> {
    use config::Config;
    use dirs::home_dir;
    use local_db::LocalDB;
    use std::sync::Arc;

    logger::setup_logger();

    let Config {
        port,
        memory,
        data_directory,
    } = argh::from_env();

    let db = if memory {
        sled::Config::new().temporary(true).open().unwrap()
    } else {
        let path = data_directory
            .unwrap_or_else(|| format!("{}/.wasm_exec_api", home_dir().unwrap().to_str().unwrap()));
        sled::open(path).unwrap()
    };
    let db = Arc::new(LocalDB(db));

    server::start(port, db).await
}

#[cfg(feature = "p2p")]
#[async_std::main]
async fn main() -> Result<(), anyhow::Error> {
    use anyhow::anyhow;
    use async_std::{sync::channel, task};
    use libp2p::{build_development_transport, identity, PeerId, Swarm};
    use p2p::{behaviour::MyBehaviour, service::P2pService, store};
    use std::sync::Arc;

    logger::setup_logger();

    // Create a random key for ourselves.
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());

    // Set up a an encrypted DNS-enabled TCP Transport over the Mplex protocol.
    let transport = build_development_transport(local_key)?;

    // Create a swarm to manage peers and events.
    let mut swarm = {
        let behaviour = MyBehaviour::new(local_peer_id.clone());
        Swarm::new(transport, behaviour, local_peer_id)
    };

    Swarm::listen_on(&mut swarm, "/ip4/0.0.0.0/tcp/0".parse()?)?;

    // Start server and wait for requests
    // TODO get port from CLI
    let (network_sender, network_receiver) = channel(50);

    // Start listening for network events on the p2p service
    let p2p = task::spawn(
        P2pService {
            swarm,
            network_receiver,
        }
        .run(),
    );

    server::start(4000, Arc::new(store::P2pStore(network_sender)))
        .await
        .map_err(|e| anyhow!("{}", e))?;

    p2p.cancel().await;

    Ok(())
}
