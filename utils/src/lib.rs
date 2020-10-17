extern crate serde;

pub mod wasm;

use serde_tuple::{Deserialize_tuple, Serialize_tuple};
use std::borrow::Cow;
use std::error::Error;
use wasmer_runtime::{instantiate, ImportObject, Instance};

/// Data layout for a wasm module.
#[derive(Serialize_tuple, Deserialize_tuple)]
pub struct WasmModule {
    /// Wasm code bytes.
    pub code: Vec<u8>,
    /// Vector of dependency module names.
    pub host_modules: Vec<String>,
}

#[derive(Serialize_tuple)]
pub struct WasmModuleRef<'a, 'm> {
    pub code: &'a [u8],
    pub host_modules: &'a [Cow<'m, str>],
}

/// Interface to allow wasm modules to be loaded and stored with different backends.
pub trait WasmStore {
    /// Loads Wasm module from store.
    fn load_module(&self, name: impl AsRef<[u8]>) -> Result<WasmModule, String>;

    /// Checks if module already exists in the store.
    fn contains_module(&self, name: impl AsRef<[u8]>) -> Result<bool, String>;

    /// Stores wasm module in store.
    fn put_module(
        &self,
        name: impl AsRef<[u8]>,
        code: &[u8],
        host_modules: &[Cow<'_, str>],
    ) -> Result<(), String>;
}

/// Loads wasm module from store, as well as loading all module dependencies recursively.
pub fn load_wasm_module_recursive<S>(db: &S, module_name: &str) -> Result<Instance, Box<dyn Error>>
where
    S: WasmStore,
{
    let module = db.load_module(module_name)?;

    let mut imports = ImportObject::new();
    for sub_module in module.host_modules {
        let loaded = load_wasm_module_recursive(db, sub_module.as_ref())?;
        imports.register(sub_module, loaded);
    }
    Ok(instantiate(module.code.as_ref(), &imports)?)
}

/// Stores wasm module to the database. This function also checks to make sure all of the
/// dependency modules exist in the database before storing the code.
pub fn store_wasm_module<S>(
    db: &S,
    module_name: &str,
    code: &[u8],
    host_modules: &[Cow<'_, str>],
) -> Result<(), Box<dyn Error>>
where
    S: WasmStore,
{
    // This check is just to short circuit the other logic, the insertion is unique.
    if db.contains_module(module_name)? {
        return Err(format!(
            "Could not store module {}: already exists in database",
            module_name
        )
        .into());
    }

    for module in host_modules {
        if !db.contains_module(module.as_bytes())? {
            return Err(format!(
                "Could not store module {}: dependency module {} does not exist in database",
                module_name, module
            )
            .into());
        }
    }

    db.put_module(module_name, code, host_modules)?;

    Ok(())
}
