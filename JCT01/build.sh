cargo build --release --target wasm32-unknown-unknown
rm -rf ../tests/wasm/JCT01.wasm
cp ./target/wasm32-unknown-unknown/release/JCT01.wasm ../tests/wasm
