FROM alpine:latest
EXPOSE 1234
WORKDIR /root/
COPY server .

CMD ["./server", "server -n test-server -a :1234"]