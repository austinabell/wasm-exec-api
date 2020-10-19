use super::utils::*;
use anyhow::{anyhow, Error};
use serde_cbor::{from_slice, to_vec};
use sled::Db;
use std::borrow::Cow;

/// Represents a sled db to load and store Wasm code.
pub struct LocalDB(pub Db);
impl WasmStore for LocalDB {
    fn load_module(&self, name: &str) -> Result<WasmModule, Error> {
        let bytes = self.0.get(name)?.ok_or_else(|| {
            anyhow!(
                "Could not find module {} in the database",
                // TODO this may not always be utf8 in future
                String::from_utf8_lossy(name.as_ref())
            )
        })?;
        Ok(from_slice(bytes.as_ref())?)
    }
    fn contains_module(&self, name: &str) -> Result<bool, Error> {
        Ok(self.0.contains_key(name)?)
    }
    fn put_module(
        &self,
        name: &str,
        code: &[u8],
        host_modules: &[Cow<'_, str>],
    ) -> Result<(), Error> {
        let serialized = to_vec(&WasmModuleRef { code, host_modules })?;
        // Compare and swap to do unique insertion to enforce modules can't be overwritten
        // with race condition.
        self.0
            .compare_and_swap(name, None as Option<&[u8]>, Some(serialized))??;
        Ok(())
    }
}
