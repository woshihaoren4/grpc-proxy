# grpc-proxy
> The grpc dynamic proxy is based on rust

## quick start
1. run a grpc service with reflection. You can use the following command to run a ready-made test program. The mac development environment is recommended.
```shell
./example/helloworld server
```
proto file following
```protobuf
service HelloWorldService {
  rpc HelloWorld(HelloWorldRequest) returns (HelloWorldResponse){
    option (google.api.http) = {
      post: "/api/v2/hello"
      body: "*"
    };
  };
}
```
2. set name and address of the grpc service to the configuration file `./src/config/config.toml`
   example：
```toml
[[proxy_sink]]
name = "hello"
addr = "127.0.0.1:8888"
```
3. run the following command
```shell
cargo run -- run
```
4. test
```shell
curl --location --request POST 'http://127.0.0.1:6789/api/v2/hello' \
--header 'Content-Type: application/json' \
--data-raw '{
    "request": "hello"
}'

```