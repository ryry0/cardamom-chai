ALL_TARGET=debug

.PHONY:: run fmt lint check test ci

all: $(ALL_TARGET)


debug:
	cargo build

release:
	cargo build --release

test:
	cargo test

run:
	cargo run -- $(filter-out $@, $(MAKECMDGOALS))

lint:
	cargo clippy

fmt:
	cargo fmt

ci: fmt lint check test

check:
	cargo check

proper:
	cargo clean

tree:
	cargo tree
