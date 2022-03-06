### Build

```
$ sudo docker image build -t simple-bitcoin/server:latest -f Dockerfile.server .
$ sudo docker image build -t simple-bitcoin/client:latest -f Dockerfile.client .
```

### Run

```
$ docker-compose up -d
```
