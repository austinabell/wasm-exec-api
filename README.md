# wasm-exec-api

Server that sets up a wasm runtime on request with hex dump of wasm binary and function call and executes

```
curl -X POST --data '{"wasm_hex": "0061736d0100000001060160017f017f030201000707010372756e00000a0601040020000b", "function_name": "run", "params": {"i32": 2}, "return_type": "i32"}' -H "Content-Type: application/json" http://localhost:8080/
```

## Generating hex dump of wasm file

```
xxd -ps -c 100000 file.wasm
```

## Using wat files

Convert using <!add link here>

```
wat2wasm file.wat -o file.wasm
```

or can dump binary and format to hex manually

```
wat2wasm file.wat --dump-module
```
