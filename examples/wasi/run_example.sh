cd functions
cargo wasi build --release
cp target/wasm32-wasi/release/functions.wasm ../
cd ..


cd runner
cargo wasi build --release
cp target/wasm32-wasi/release/runner.wasm ../
cd ..

wat2wasm text.wat -o text.wasm

# Register wasm files on server
curl -X POST --data '{"module_name": "text", "wasm_hex": "'$(echo -n $(xxd -ps -c 100000 text.wasm))'"}' -H "Content-Type: application/json" http://localhost:4000/register
curl -X POST --data '{"module_name": "functions", "wasm_hex": "'$(echo -n $(xxd -ps -c 100000 functions.wasm))'"}' -H "Content-Type: application/json" http://localhost:4000/register
# curl -X POST --data '{"module_name": "runner", "wasm_hex": "'$(echo -n $(xxd -ps -c 100000 runner.wasm))'", "host_modules": ["text", "functions"]}' -H "Content-Type: application/json" http://localhost:4000/register

# Check sub functions
curl -X POST --data '{"module_name": "text", "function_name": "double", "params": [3]}' -H "Content-Type: application/json" http://localhost:4000/execute
curl -X POST --data '{"module_name": "functions", "function_name": "fibonacci", "params": [4]}' -H "Content-Type: application/json" http://localhost:4000/execute

# Run function to call sub modules
# curl -X POST --data '{"module_name": "runner", "function_name": "_start"}' -H "Content-Type: application/json" http://localhost:4000/execute
