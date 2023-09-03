all: build

help:
	@echo ""
	@echo "make build             - Build binary files"
	@echo "make install           - Install binary files"
	@echo "make rust              - Install rust"
	@echo "make precommit         - Execute precommit steps"
	@echo "make loc               - Count lines of code in src folder"
	@echo ""

build:
	cargo build --release

install:
	sudo cp ./target/release/reverseproxy /usr/local/bin
	chmod a+x /usr/local/bin/reverseproxy

rust:
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

precommit:
	cargo fmt && cargo clippy
	cargo clippy --no-default-features

clean:
	cargo clean

loc:
	@echo "--- Counting lines of .rs files (LOC):" && find src/ -type f -name "*.rs" -exec cat {} \; | wc -l
