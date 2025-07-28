
### to install tools
cargo install cargo-watch
cargo install trunk
rustup target add wasm32-unknown-unknown


#### In one terminal
cd app; trunk watch 

#### In another terminal
cargo watch -w server -w src -x "run"



#### To run on server
cargo run
