# should be ran from project root.
cd payload 
cargo build 
cd ../runner
cargo build 
sleep 1
cargo run 