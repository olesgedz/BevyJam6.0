# BevyJam6.0

## How to build wasm and run
# Prerequisites 
rustup target add wasm32-unknown-unknown

cargo install trunk

# Build and run on the port
trunk serve --open --port 9000

# Build for release
trunk build --release
