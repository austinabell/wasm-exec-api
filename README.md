# wasm-exec-api

**This is wip and just as poc/fun for now** 

Server that sets up a wasm runtime on request with hex dump of wasm binary and function call and executes

Call wasm code with hex dump of wasm file and function name. Params and return type are optional.
Example:

```
curl -X POST --data '{"wasm_hex": "0061736d0100000001060160017f017f030201000707010372756e00000a0601040020000b", "function_name": "run", "params": {"i32": 2}, "return_type": "i32"}' -H "Content-Type: application/json" http://localhost:8080/
```

## Generating hex dump of wasm file

Convert wasm binary file to hex dump, which will be used as `"wasm_hex"` parameter
```
xxd -ps -c 100000 file.wasm
```

## Using wat files

Convert using [WASM binary toolkit](https://github.com/WebAssembly/wabt) `wat2wasm`

```
wat2wasm file.wat -o file.wasm
```

and then use `xxd` to convert to hex, or can dump binary and format to hex manually

```
wat2wasm file.wat --dump-module
```
