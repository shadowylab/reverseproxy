# TCP Reverse Proxy written in Rust

## Getting started 

### Install from [crates.io](https://crates.io/crates/reverseproxy)

```
cargo install reverseproxy
```

### Install from source

* [Build from source](doc/build.md) 

## Usage

* General usage

    ```shell
    reverseproxy <protocol>://<address> <protocol>://<address>
    ```

* Forward from local address to Tor hidden service (.onion) using socks5 proxy

    ```shell
    reverseproxy tcp://127.0.0.1:8080 tcp://torhiddenservice.onion:80 --socks5-proxy 127.0.0.1:9050
    ```

* Forward from local address to local network address

    ```shell
    reverseproxy tcp://127.0.0.1:8080 tcp://othercomputer.local:80 
    ```

* Forward from local address to Tor hidden service (.onion) using embedded Tor client

    ```shell
    reverseproxy tcp://127.0.0.1:8080 tcp://torhiddenservice.onion:80 --tor
    ```

* Forward from local address to UNIX socket

    ```shell
    reverseproxy tcp://127.0.0.1:8080 unix:///tmp/temp.sock
    ```

To get more info use `reverseproxy --help`

## License

This project is distributed under the MIT software license - see the [LICENSE](LICENSE) file for details
