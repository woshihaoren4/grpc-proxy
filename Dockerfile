FROM alpine:latest
EXPOSE 6789
WORKDIR /root/
COPY target/x86_64-unknown-linux-musl/release/rust-grpc-proxy .

CMD ["./rust-grpc-proxy", "run"]
