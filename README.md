# TCP Reverse Proxy written in Rust

## Install 

* [Build from source](doc/build.md) 

## Usage

Does NOT support TLS yet!

* Forward to Tor hidden service (.onion) using socks5 proxy

```
reverseproxy --server 127.0.0.1:8080 --forward torhiddenservice.onion:80 --socks5-proxy 127.0.0.1:9050
```

* Forward to local network address

```
reverseproxy --server 127.0.0.1:8080 --forward othercomputer.local:80 
```

## License

This project is distributed under the MIT software license - see the [LICENSE](LICENSE) file for details