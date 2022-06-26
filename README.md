# TCP Reverse Proxy written in Rust

## Install 

1. Download the source code

```
git clone https://gitlab.com/p2kishimoto/reverseproxy.git
```

2. Install dependencies

* Ubuntu & Debian

```
sudo apt install libssl-dev make pkg-config
```

2. Install Rust (skip if you already have Rust installed)

```
make rust
```

3. Compile source code

```
make
```

4. Install (copy binary file to `/usr/local/bin`)

```
sudo make install
```

## Usage

* Forward to Tor hidden service (.onion) using `default` socks5 proxy

```
reverseproxy --server 127.0.0.1:8080 --forward torhiddenservice.onion:80 --use-tor
```

* Forward to Tor hidden service (.onion) using `custom` socks5 proxy

```
reverseproxy --server 127.0.0.1:8080 --forward torhiddenservice.onion:80 --socks5-proxy 127.0.0.1:9050
```

## License

This project is distributed under the MIT software license - see the [LICENSE](LICENSE) file for details