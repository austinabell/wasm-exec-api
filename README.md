# wasm-exec-api

**This is wip and just as poc/fun for now** 

Server that sets up a wasm runtime on request with hex dump of wasm binary, function name to call, and optionally parameters for the function call and executes.

### Example request

```
curl -X POST --data '{"wasm_hex": "0061736d0100000001060160017f017f03020100070a0106646f75626c6500000a09010700200041026c0b", "function_name": "double", "params": [2]}' -H "Content-Type: application/json" http://localhost:4000/
```

## Next steps

- [x] Remote code execution on server
- [x] Dynamic function calls to not have to specify return type and manually handle cases
- [x] Wasm code inspection to be able to infer the params types to not have to specify the enum of possible values (example: `2` instead of `{"I32": 2}`) for each value and give better error returns
- [ ] Functionality to set shared memory of runtime environment to be able to be used in func
- [x] Implement way to setup host functions to access during execution

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

## Registering host functions

```
# Register the function to be used
curl -X POST --data '{"module_name": "utils", "wasm_hex": "0061736d0100000001060160017f017f03020100070a0106646f75626c6500000a09010700200041026c0b"}' -H "Content-Type: application/json" http://localhost:4000/register

# Call the function, including the module import
curl -X POST --data '{"wasm_hex": "0061736d0100000001060160017f017f021001057574696c7306646f75626c650000030201000710010c646f75626c655f747769636500010a0a0108002000100010000b", "function_name": "double_twice", "params": [2], "host_modules": ["utils"]}' -H "Content-Type: application/json" http://localhost:4000/
```
