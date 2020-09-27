use serde_cbor::{from_slice, to_vec};
use serde_tuple::{Deserialize_tuple, Serialize_tuple};
use sled::Db;
use std::borrow::Cow;
use std::error::Error;
use wasmer_runtime::{instantiate, ImportObject, Instance};

#[derive(Serialize_tuple, Deserialize_tuple)]
pub struct WasmModule<'a> {
    // Wasm code bytes
    code: Vec<u8>,
    host_modules: Vec<Cow<'a, str>>,
}

#[derive(Serialize_tuple)]
pub struct WasmModuleRef<'a, 'm> {
    // Wasm code bytes
    code: &'a [u8],
    host_modules: &'a [Cow<'m, str>],
}

/// Loads wasm module from store, as well as loading all module dependencies recursively.
pub fn load_wasm_module_recursive(db: &Db, module_name: &str) -> Result<Instance, Box<dyn Error>> {
    let bytes = db
        .get(module_name)?
        .ok_or_else(|| format!("Could not find module {} in the database", module_name))?;
    let module: WasmModule = from_slice(bytes.as_ref())?;

    let mut imports = ImportObject::new();
    for sub_module in module.host_modules {
        let loaded = load_wasm_module_recursive(db, sub_module.as_ref())?;
        imports.register(sub_module, loaded);
    }
    Ok(instantiate(module.code.as_ref(), &imports)?)
}

/// Stores wasm module to the database. This function also checks to make sure all of the
/// dependency modules exist in the database before storing the code.
pub fn store_wasm_module(
    db: &Db,
    module_name: &str,
    code: &[u8],
    host_modules: &[Cow<'_, str>],
) -> Result<(), Box<dyn Error>> {
    // This check is just to short circuit the other logic, the insertion is unique.
    if db.contains_key(module_name)? {
        return Err(format!(
            "Could not store module {}: already exists in database",
            module_name
        )
        .into());
    }

    for module in host_modules {
        if !db.contains_key(module.as_bytes())? {
            return Err(format!(
                "Could not store module {}: dependency module {} does not exist in database",
                module_name, module
            )
            .into());
        }
    }

    let serialized = to_vec(&WasmModuleRef { code, host_modules })?;
    // Compare and swap to do unique insertion to enforce modules can't be overwritten
    // with race condition.
    db.compare_and_swap(module_name, None as Option<&[u8]>, Some(serialized))??;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sled::Config;

    #[test]
    fn wasm_module_symmetric_serialize() {
        let wasm_ref = WasmModuleRef {
            code: b"test code",
            host_modules: &["one".into(), "two".into()],
        };
        let serialized = to_vec(&wasm_ref).unwrap();
        let wasm_mod_deser: WasmModule = from_slice(&serialized).unwrap();
        assert_eq!(wasm_mod_deser.code, wasm_ref.code);
        assert_eq!(wasm_mod_deser.host_modules, wasm_ref.host_modules);
    }

    #[test]
    fn store_load() {
        let config = Config::new().temporary(true);
        let db = config.open().unwrap();
        let code = include_bytes!("../utils.wasm");

        assert!(load_wasm_module_recursive(&db, "utils").is_err());

        // Trying to load with dependency module that doesn't exist
        assert!(store_wasm_module(&db, "test", code, &["utils".into()]).is_err());

        // Store and load utils
        store_wasm_module(&db, "utils", code, &[]).unwrap();
        assert!(load_wasm_module_recursive(&db, "utils").is_ok());

        // Shouldn't be able to overwrite existing module
        assert!(store_wasm_module(&db, "utils", code, &[]).is_err());

        // Should be able to store link with host module of now stored "utils"
        store_wasm_module(&db, "link", code, &["utils".into()]).unwrap();
        assert!(load_wasm_module_recursive(&db, "link").is_ok());
    }
}
