# HTTP/1.x Reverse Proxy written in Rust

## Install 

1. Download the source code

```
git clone https://gitlab.com/pskishimoto/reverseproxy.git
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

* Forward to Tor hidden service (.onion) using socks5 proxy

```
torproxy --server 127.0.0.1:8080 --forward http://torhiddenservice.onion --proxy socks5h://127.0.0.1:9050
```

## License

This project is distributed under the MIT software license - see the [LICENSE](LICENSE) file for details