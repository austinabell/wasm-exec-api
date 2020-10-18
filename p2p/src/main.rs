#![recursion_limit = "1024"]

mod behaviour;
mod logger;
mod service;
mod store;

use anyhow::{anyhow, Error};
use async_std::{sync::channel, task};
use behaviour::MyBehaviour;
use libp2p::{build_development_transport, identity, PeerId, Swarm};
use service::P2pService;
use std::sync::Arc;
use wasm_exec_api::server;

#[async_std::main]
async fn main() -> Result<(), Error> {
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
