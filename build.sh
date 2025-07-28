

cargo install cargo-watch
cargo install trunk
rustup target add wasm32-unknown-unknown

cd app; trunk watch #cargo build

#cd src; cargo build

cargo watch -w server -w src -x "run"


#unsure of order. app first?

cargo run
