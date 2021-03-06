wasm-exec-api
=======

[<img alt="build status" src="https://img.shields.io/github/workflow/status/austinabell/wasm-exec-api/CI/main?style=for-the-badge" height="20">](https://github.com/austinabell/wasm-exec-api/actions?query=branch%3Amain)

**This is just a fun project and should not be used for any production use cases**

Wasm arbitrary code execution through a remote REST API server. Wasm code can be executed through a `POST` request to the `/` endpoint with hex encoded wasm binary, function name to call, and optionally parameters to pass in the function.

Code can also be persisted and recursively linked, using the `/register` endpoint, which accepts a `POST` request with the module name, code, and optionally sub modules to be accessible by the module. With registered functions, requests can be sent to `/execute` to call a function with parameters from the registered module.

### Example request

```bash
curl -X POST --data '{"function_name": "double", "params": [2], "wasm_hex": "0061736d0100000001060160017f017f03020100070a0106646f75626c6500000a09010700200041026c0b"}' -H "Content-Type: application/json" http://localhost:4000/
```

## Next steps

- [x] Arbitrary code execution on server
- [x] Dynamic function calls to not have to specify return type and manually handle cases
- [x] Wasm code inspection to be able to infer the params types to not have to specify the enum of possible values (example: `2` instead of `{"I32": 2}`) for each value and give better error returns
- [ ] Functionality to set shared memory of runtime environment to be able to be used in func
- [x] Implement way to setup host functions to access during execution
- [x] Storing Wasm modules on a distributed hash table on a p2p network

## Generating hex dump of wasm file

Convert wasm binary file to hex dump, which will be used as `"wasm_hex"` parameter
```bash
xxd -ps -c 100000 file.wasm
```

## Using wat files

Convert using [WASM binary toolkit](https://github.com/WebAssembly/wabt) `wat2wasm`

```bash
wat2wasm file.wat -o file.wasm
```

and then use `xxd` to convert to hex, or can dump binary and format to hex manually

```bash
wat2wasm file.wat --dump-module
```

## Registering host modules

```bash
# Register the function to be used
curl -X POST --data '{"module_name": "utils", "wasm_hex": "0061736d0100000001060160017f017f03020100070a0106646f75626c6500000a09010700200041026c0b"}' -H "Content-Type: application/json" http://localhost:4000/register

# This registered module can be called directly with the execute endpoint
curl -X POST --data '{"module_name": "utils", "function_name": "double", "params": [2]}' -H "Content-Type: application/json" http://localhost:4000/execute

# Can use general endpoint to execute code that uses the registered module as a host function
curl -X POST --data '{"wasm_hex": "0061736d0100000001060160017f017f021001057574696c7306646f75626c650000030201000710010c646f75626c655f747769636500010a0a0108002000100010000b", "function_name": "double_twice", "params": [2], "host_modules": ["utils"]}' -H "Content-Type: application/json" http://localhost:4000/
```

## Wasm module store backends

The default backend when running the API is a [sled](https://github.com/spacejam/sled) database. The data directory can be configured or can be replaced with an in memory store.

There is an alternative backend which starts a peer to peer node with [libp2p](https://github.com/libp2p/rust-libp2p) and uses a Kademlia distributed hash table (DHT). This alternative is not as stable as the regular client-server architecture, but it's more fun.

To run as a p2p node, compile with the `p2p` feature:
```bash
cargo build --release --features p2p
./target/release/wasm-exec-api
```
