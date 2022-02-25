### Build

```
$ cargo build
```

### Run

```
$ ./target/debug/server -l 127.0.0.1:20011
```

On another console:

```
$ ./target/debug/server -l 127.0.0.1:20012 -c 127.0.0.1:20011
```
