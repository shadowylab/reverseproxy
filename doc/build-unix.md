# BUILD FOR UNIX

## Introduction

Before build, see [build requirements](#build-requirements) for your specific platform.

## Install Rust

If you have already installed Rust on your device you can skip to the next step, otherwise:

```
make rust
```

## Build

Compile binary from source code:

```
make
```

Optional, install binary in `/usr/local/bin`:

```
sudo make install
```

## Build requirements

### Ubuntu & Debian

```
sudo apt install build-essential libssl-dev make pkg-config automake autoconf libtool libsqlite-dev
```

### Fedora

```
sudo dnf group install "C Development Tools and Libraries" "Development Tools"
```

```
sudo dnf install make openssl-devel automake autoconf libtool
```

### MacOS

```
brew install automake pkg-config
```