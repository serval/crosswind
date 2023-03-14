#!/usr/bin/env bash
ssh mark@fd7a:115c:a1e0:ab12:4843:cd96:6247:1748 "source ~/.zshrc ; cd ~/code/serval/kaboodle ; cargo run -- --interface 'fd7a:115c:a1e0:ab12:4843:cd96:6247:1748' | grep Peers:"