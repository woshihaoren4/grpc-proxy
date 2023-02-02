# grpc-proxy
> The grpc dynamic proxy is based on rust

## quick start
1. start a grpc service with reflection,proto file following
```protobuf
syntax = "proto3";
package proto;
option go_package = "./proto";
import "google/api/annotations.proto";

service HelloWorldService {
  rpc HelloWorld(HelloWorldRequest) returns (HelloWorldResponse){
    option (google.api.http) = {
      post: "/api/v2/hello"
      body: "*"
    };
  };
}

message HelloWorldRequest {
  string request = 1;
}
message HelloWorldResponse {
  string response = 1;
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