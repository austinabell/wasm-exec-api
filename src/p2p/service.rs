use super::behaviour::MyBehaviour;
use async_std::sync::Receiver;
use futures::channel::oneshot::Sender as OneshotSender;
use futures::{select, StreamExt};
use libp2p::kad::{record::Key, Quorum, Record};
use libp2p::Swarm;

pub enum NetworkRequest {
    GetDHTKey {
        request: Key,
        response_channel: OneshotSender<Vec<u8>>,
    },
    PutDHTKey {
        key: Key,
        value: Vec<u8>,
    },
}

pub struct P2pService {
    pub swarm: Swarm<MyBehaviour>,
    pub network_receiver: Receiver<NetworkRequest>,
}

impl P2pService {
    pub async fn run(self) {
        let mut network_stream = self.network_receiver.fuse();
        let mut swarm = self.swarm.fuse();

        loop {
            select! {
                net_event = network_stream.next() => match net_event {
                    Some(NetworkRequest::GetDHTKey {
                        request,
                        response_channel,
                    }) => {
                        // Send request for the record
                        swarm.get_mut().kademlia.get_record(&request, Quorum::One);

                        // Push awaiting channel at index of the key.
                        let channels = swarm.get_mut().awaiting_response.entry(request).or_default();
                        channels.push(response_channel);
                    }
                    Some(NetworkRequest::PutDHTKey { key, value }) => {
                        // Generate a record to put in the DHT
                        let record = Record {
                            key,
                            value,
                            publisher: None,
                            expires: None,
                        };
                        swarm.get_mut().kademlia.put_record(record, Quorum::One).unwrap();
                    }
                    None => break,
                },
                swarm_event = swarm.next() => match swarm_event {
                    Some(event) => {
                        // This would be unexpected: there are no events bubbled up
                        log::warn!("{:?}", event);
                    }
                    None => break,
                }
            }
        }
    }
}
