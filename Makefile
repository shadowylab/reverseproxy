# Compiler configuration
GENERAL_ARGS = --release

DEFAULT = build

# Use 'VERBOSE=1' to echo all commands, for example 'make help VERBOSE=1'.
ifdef VERBOSE
  Q :=
else
  Q := @
endif

all: $(DEFAULT)

help:
	$(Q)echo ""
	$(Q)echo "make build             - Build binary files"
	$(Q)echo "make install           - Install binary files"
	$(Q)echo "make rust              - Install rust"
	$(Q)echo "make precommit         - Execute precommit steps"
	$(Q)echo "make loc               - Count lines of code in src folder"
	$(Q)echo ""

build:
	$(Q)cargo build $(GENERAL_ARGS)

install:
	$(Q)sudo cp ./target/release/TBD /usr/local/bin
	$(Q)chmod a+x /usr/local/bin/TBD

rust:
	$(Q)curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

precommit:
	$(Q)cargo fmt && cargo clippy

clean:
	$(Q)cargo clean

loc:
	$(Q)echo "--- Counting lines of .rs files (LOC):" && find src/ -type f -name "*.rs" -exec cat {} \; | wc -l