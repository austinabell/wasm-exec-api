use serde_cbor::{from_slice, to_vec};
use sled::Db;
use std::borrow::Cow;
use utils::*;

/// Represents a sled db to load and store Wasm code.
pub struct LocalDB(pub Db);
// TODO add async recursion, then change String error mappings
impl WasmStore for LocalDB {
    fn load_module(&self, name: impl AsRef<[u8]>) -> Result<WasmModule, String> {
        let bytes = self
            .0
            .get(name.as_ref())
            .map_err(|e| e.to_string())?
            .ok_or_else(|| {
                format!(
                    "Could not find module {} in the database",
                    // TODO this may not always be utf8 in future
                    String::from_utf8_lossy(name.as_ref())
                )
            })?;
        from_slice(bytes.as_ref()).map_err(|e| e.to_string())
    }
    fn contains_module(&self, name: impl AsRef<[u8]>) -> Result<bool, String> {
        self.0
            .contains_key(name.as_ref())
            .map_err(|e| e.to_string())
    }
    fn put_module(
        &self,
        name: impl AsRef<[u8]>,
        code: &[u8],
        host_modules: &[Cow<'_, str>],
    ) -> Result<(), String> {
        let serialized =
            to_vec(&WasmModuleRef { code, host_modules }).map_err(|e| e.to_string())?;
        // Compare and swap to do unique insertion to enforce modules can't be overwritten
        // with race condition.
        self.0
            .compare_and_swap(name.as_ref(), None as Option<&[u8]>, Some(serialized))
            .map_err(|e| e.to_string())?
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}
