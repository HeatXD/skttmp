# should be ran from project root.
cd payload 
cargo build --release
cd ../runner
cargo build --release
cargo run --release