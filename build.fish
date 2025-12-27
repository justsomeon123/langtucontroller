#!/usr/bin/env fish
cd (dirname (status current-filename))
cargo build -r
sudo mv target/release/langtuctl /usr/bin/langtuctl
