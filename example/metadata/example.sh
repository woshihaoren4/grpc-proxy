#!/bin/bash

SERVICE_ECHO="SERVICE_ECHO"
GRPC_PROXY="GRPC_PROXY"
CONFIG_PATH="config.toml"

function test_one() {
    result=$(curl -s -l --location --request GET 'http://127.0.0.1:6789/api/v1/echo/hello/get?query=666' | jq -r '."response"')
    assert_eq "$result" 'GET [SERVICE_ECHO]---> request=hello query=666' "test_one"
}
function test_three() {
    result=$(curl  -s -l --location --request POST 'http://127.0.0.1:6789/api/v1/echo/post' \
             --header 'Content-Type: application/json' \
             --data-raw '{
                 "request": "hello",
                 "query": 123
             }' | jq -r '.response')
    assert_eq "$result" 'POST [SERVICE_ECHO]---> request=hello query=123' "test_three"
}

function assert_eq() {
    if [ "$1" == "$2" ]
    then
    echo "用例[$3] success"
    else
    echo -e "===> 用例[$3] assert failed <=== "
    echo -e "        expectation -->$2 "
    echo -e "        actual      -->$1 "
    fi
}


function reset_config_file() {
cat>$CONFIG_PATH <<EOF
[log]
show_file_line = false

[[proxy_sink]]
name = "echo"
addr = "127.0.0.1:1234"
prefix = "/"

[metadata_filters]
prefix = ["echo-","greet-"]
match = ["use-name","use-id"]
EOF
}

function start_server() {
    if [ $(screen -ls | grep -c $SERVICE_ECHO) -le 0 ]
    then
      screen -dmS $SERVICE_ECHO /bin/bash -c "./server server -n $SERVICE_ECHO -a :1234" && echo "测试服务[$SERVICE_ECHO]已启动"
    else
      echo "$SERVICE_ECHO 服务已经启动"
    fi

}

case $1 in
clean)
  kill -2 $(lsof -i:6789 | grep rust-grpc | awk '{print $2}')
  kill -2 $(lsof -i:1234 | grep server | awk '{print $2}')
  screen -R $GRPC_PROXY -X quit ; echo "$GRPC_PROXY 已清理"
  screen -R $SERVICE_ECHO -X quit ; echo "$SERVICE_ECHO 已清理"
  rm $CONFIG_PATH ; echo "配置文件[$CONFIG_PATH] 已清理"
  exit
  ;;
config)
  reset_config_file
  echo "reset config file ---> $CONFIG_PATH" && cat $CONFIG_PATH
  exit
  ;;
server)
  start_server && echo 'src server run success'
  exit
esac



start_server


if [ ! -e $CONFIG_PATH ]
then
  reset_config_file && echo "配置文件：$CONFIG_PATH 初始化成功"
fi

if [ $(screen -ls | grep -c $GRPC_PROXY) -le 0 ]
then
  echo -n "等待测试服务启动"
  for (( i=0;i<10;i=i+1 ))
  do
    echo -n "。"
    sleep 1
  done
  echo ""

  screen -dmS $GRPC_PROXY /bin/bash -c "./rust-grpc-proxy run -c $CONFIG_PATH" && echo "代理服务[$GRPC_PROXY]已启动"

    echo -n "等待代理服务启动"
    for (( i=0;i<10;i=i+1 ))
    do
      echo -n "。"
      sleep 1
    done
    echo ""
else
  echo "$GRPC_PROXY 服务已经启动"
fi

if [ ! -x "$(command -v jq)" ];then
  echo 'jq not found , please install'
  echo '    MAC    : brew install jq'
  echo '    Ubuntu : sudo apt-get install jq'
  exit 1
fi

test_one

test_three