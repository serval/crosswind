#!/usr/bin/env bash
cd ~/code/serval/kaboodle
cargo run -- --interface 'fd7a:115c:a1e0:ab12:4843:cd96:6250:6d45'| tee | grep Peers:
