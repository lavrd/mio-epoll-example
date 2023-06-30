build:
	cargo build

run: build
	RUST_LOG=trace RUST_BACKTRACE=1 ./target/debug/mio-epoll-example

lint:
	cargo clippy --tests --workspace -- -D warnings
