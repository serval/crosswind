#!/usr/bin/env bash
echo "Press any key to start crosswind (nothing will work until you do this)"
read
cargo run -- --interface utun3 --multicast-address '[ff02::1213:1989]:7475' --targets '[fd7a:115c:a1e0:ab12:4843:cd96:6247:1748]:9908'
