#!/bin/bash

case $1 in
run)
cargo run -- run
  ;;
show)
cargo run -- show
esac