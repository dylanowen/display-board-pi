SHELL:=/bin/bash

.DEFAULT_GOAL: build
.PHONY: fix fmt lint check build release test dev dev-trace dev-release simulator pre-commit clean

target = arm-unknown-linux-musleabihf
binary = display-board-pi

fix:
	cargo fix --allow-staged --all-targets

fmt:
	cargo fmt --all -- --check

lint:
	cargo clippy --all-targets --fix --allow-staged -- -D warnings
	cargo clippy --all-targets --no-default-features --features=max-simulator --fix --allow-staged -- -D warnings
	-cargo audit

check:
	cargo check --target $(target)

build:
	cross build --target $(target)

release:
	cross build --release --target $(target)

test:
	cargo test

dev: build
	scp target/$(target)/debug/$(binary) pi@raspberrypi.local:~/
	ssh -t pi@raspberrypi.local 'RUST_BACKTRACE=1 RUST_LOG=debug ./$(binary)'

dev-trace: build
	scp target/$(target)/debug/$(binary) pi@raspberrypi.local:~/
	ssh -t pi@raspberrypi.local 'RUST_BACKTRACE=1 RUST_LOG=trace ./$(binary)'

dev-release: release
	scp target/$(target)/release/$(binary) pi@raspberrypi.local:~/
	ssh -t pi@raspberrypi.local 'RUST_LOG=debug ./$(binary)'

simulator:
	cargo run --no-default-features --features=max-simulator

pre-commit: fmt lint test release

clean:
	cargo clean