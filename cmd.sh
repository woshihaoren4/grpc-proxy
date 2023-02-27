#!/bin/bash

case $1 in
run)
cargo run -- run
  ;;
show)
cargo run -- show
  ;;
restful)
cargo run -- run -c example/restful/config.toml
  ;;
md)
cargo run -- run -c example/metadata/config.toml
  ;;
docker)

if [ ! -e ".cargo/config.toml" ] ; then
  mkdir .cargo;touch .cargo/config.toml
fi

cat>".cargo/config.toml" <<EOF
[target.x86_64-unknown-linux-musl]
linker = "x86_64-linux-musl-gcc"
EOF

cargo build --release --target=x86_64-unknown-linux-musl
chmod +x target/x86_64-unknown-linux-musl/release/rust-grpc-proxy
tag="wdshihaoren/rust-grpc-proxy:v0.0.2-t"
docker build -f ./Dockerfile -t "$tag"  .
#  docker pull "wdshihaoren/rust-grpc-proxy
rm -rf .cargo
  ;;
*)
  echo "unknown cmd \"${1}\""
  ;;
esac

