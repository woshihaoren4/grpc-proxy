#!/bin/bash

TAG="registry.cn-hangzhou.aliyuncs.com/wshr/wd:latest"
docker build -f ./Dockerfile -t "$TAG" .
docker push "$TAG"