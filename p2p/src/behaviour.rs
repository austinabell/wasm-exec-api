use futures::channel::oneshot::Sender as OneshotSender;
use libp2p::kad::record::store::MemoryStore;
use libp2p::kad::{record::Key, Kademlia, Record};
use libp2p::kad::{KademliaEvent, PeerRecord, PutRecordOk, QueryResult};
use libp2p::swarm::NetworkBehaviourEventProcess;
use libp2p::{
    mdns::{Mdns, MdnsEvent},
    NetworkBehaviour, PeerId,
};
use std::collections::HashMap;

#[derive(NetworkBehaviour)]
pub struct MyBehaviour {
    pub kademlia: Kademlia<MemoryStore>,
    pub mdns: Mdns,
    #[behaviour(ignore)]
    pub awaiting_response: HashMap<Key, Vec<OneshotSender<Vec<u8>>>>,
}

impl NetworkBehaviourEventProcess<MdnsEvent> for MyBehaviour {
    fn inject_event(&mut self, event: MdnsEvent) {
        if let MdnsEvent::Discovered(list) = event {
            for (peer_id, multiaddr) in list {
                self.kademlia.add_address(&peer_id, multiaddr);
            }
        }
    }
}

impl NetworkBehaviourEventProcess<KademliaEvent> for MyBehaviour {
    fn inject_event(&mut self, message: KademliaEvent) {
        match message {
            KademliaEvent::QueryResult { result, .. } => match result {
                QueryResult::GetRecord(Ok(ok)) => {
                    for PeerRecord {
                        record: Record { key, value, .. },
                        ..
                    } in ok.records
                    {
                        if let Some(responses) = self.awaiting_response.remove(&key) {
                            for r in responses {
                                let _ = r.send(value.clone());
                            }
                        }
                        log::info!("Received {}", String::from_utf8_lossy(key.as_ref()));
                    }
                }
                QueryResult::GetRecord(Err(err)) => {
                    log::warn!("Failed to get record: {:?}", err);
                }
                QueryResult::PutRecord(Ok(PutRecordOk { key })) => {
                    log::info!(
                        "successfully put key: {}",
                        String::from_utf8_lossy(key.as_ref())
                    );
                }
                QueryResult::PutRecord(Err(err)) => {
                    log::warn!("Failed to put record: {:?}", err);
                }
                _ => {}
            },
            _ => {}
        }
    }
}

impl MyBehaviour {
    pub fn new(local_peer_id: PeerId) -> Self {
        let store = MemoryStore::new(local_peer_id.clone());
        let kademlia = Kademlia::new(local_peer_id, store);
        let mdns = Mdns::new().unwrap();
        Self {
            kademlia,
            mdns,
            awaiting_response: Default::default(),
        }
    }
}
