 cargo build
 wasm-bindgen --out-dir target/generated --web target/wasm32-unknown-unknown/debug/rs_wgpu_cube.wasm
 simple-http-server target/generated --ip 127.0.0.1 -p 301