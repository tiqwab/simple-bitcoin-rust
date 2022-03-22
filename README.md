Rust implementation of Simple Bitcoin, which is a simple blockchain developed in [ゼロから創る暗号通貨](https://peaks.cc/books/cryptocurrency).

### Run

```
# Start containers
$ docker-compose up -d --build server1
$ docker-compose up -d --build server2
$ docker-compose up -d --build client1 client2
$ docker-compose up -d --build ui1 ui2

# Generate coins for client1
# (this is for test and should be executed only once)
$ curl -XPOST http://localhost:30013/generate-block
```

Two WebUIs are available on http://localhost:8081 and http://localhost:8082.
