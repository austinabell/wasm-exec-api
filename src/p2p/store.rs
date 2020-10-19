use super::service::NetworkRequest;
use crate::utils::{WasmModule, WasmModuleRef, WasmStore};
use anyhow::Error;
use async_std::future;
use async_std::{sync::Sender, task};
use futures::channel::oneshot;
use libp2p::kad::record::Key;
use serde_cbor::{from_slice, to_vec};
use std::borrow::Cow;
use std::time::Duration;

/// Represents a sled db to load and store Wasm code.
pub struct P2pStore(pub Sender<NetworkRequest>);
impl WasmStore for P2pStore {
    fn load_module(&self, name: &str) -> Result<WasmModule, Error> {
        let bytes = task::block_on(async {
            let (tx, rx) = oneshot::channel();
            self.0
                .send(NetworkRequest::GetDHTKey {
                    request: Key::new(&name),
                    response_channel: tx,
                })
                .await;
            future::timeout(Duration::from_secs(3), rx).await
        })??;

        Ok(from_slice(bytes.as_ref())?)
    }
    fn contains_module(&self, name: &str) -> Result<bool, Error> {
        task::block_on(async {
            let (tx, rx) = oneshot::channel();
            self.0
                .send(NetworkRequest::GetDHTKey {
                    request: Key::new(&name),
                    response_channel: tx,
                })
                .await;
            match future::timeout(Duration::from_secs(2), rx).await {
                Err(_) => Ok(false),
                Ok(Ok(_)) => Ok(true),
                Ok(Err(e)) => Err(e.into()),
            }
        })
    }
    fn put_module(
        &self,
        name: &str,
        code: &[u8],
        host_modules: &[Cow<'_, str>],
    ) -> Result<(), Error> {
        let value = to_vec(&WasmModuleRef { code, host_modules })?;

        task::block_on(self.0.send(NetworkRequest::PutDHTKey {
            key: Key::new(&name),
            value,
        }));
        Ok(())
    }
}
