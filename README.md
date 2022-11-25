# TCP Reverse Proxy written in Rust

## Getting started 

### Install from [crates.io](https://crates.io/crates/reverseproxy)

```
cargo install reverseproxy
```

### Install from source

* [Build from source](doc/build.md) 

## Usage

Does NOT support TLS yet!

* Forward from local address to Tor hidden service (.onion) using socks5 proxy

    ```
    reverseproxy 127.0.0.1:8080 torhiddenservice.onion:80 --socks5-proxy 127.0.0.1:9050
    ```

* Forward from local address to local network address

    ```
    reverseproxy 127.0.0.1:8080 othercomputer.local:80 
    ```

To get more info use `reverseproxy --help`

### Experimental

* Forward from local address to Tor hidden service (.onion) using embedded Tor client

    To enable this feature, build the binary using `cargo build --release --features tor` or `cargo install --features tor`

    ```
    reverseproxy 127.0.0.1:8080 torhiddenservice.onion:80 --use-tor
    ```

## License

This project is distributed under the MIT software license - see the [LICENSE](LICENSE) file for details