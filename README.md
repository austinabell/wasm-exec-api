# wasm-exec-api

Server that sets up a wasm runtime on request with hex dump of wasm binary and function call and executes


wat2wasm file.wat --dump-module
xxd file.wasm

curl -X POST --data '{...}' -H "Content-Type: application/json" http://localhost:8088/
