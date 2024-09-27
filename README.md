# TCP Reverse Proxy written in Rust

## Getting started 

### Install from [crates.io](https://crates.io/crates/reverseproxy)

```
cargo install reverseproxy
```

### Install from source

* [Build from source](doc/build.md) 

## Usage

* Forward from local address to Tor hidden service (.onion) using socks5 proxy

    ```shell
    reverseproxy 127.0.0.1:8080 torhiddenservice.onion:80 --socks5-proxy 127.0.0.1:9050
    ```

* Forward from local address to local network address

    ```shell
    reverseproxy 127.0.0.1:8080 othercomputer.local:80 
    ```

* Forward from local address to Tor hidden service (.onion) using embedded Tor client

    ```shell
    reverseproxy 127.0.0.1:8080 torhiddenservice.onion:80 --tor
    ```

To get more info use `reverseproxy --help`

## License

This project is distributed under the MIT software license - see the [LICENSE](LICENSE) file for details
