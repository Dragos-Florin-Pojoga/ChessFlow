# Note: this script assumes rust will handle the incremental builds

all: toolchain build_native build_web


toolchain:
	rustup toolchain install nightly
	rustup component add rust-src --toolchain nightly
	cargo install wasm-bindgen-cli
	cargo install wasm-pack
	rustup target add wasm32-unknown-unknown


build_native:
	cargo build --bin native_engine --release

build_web:
	wasm-pack build -t no-modules --out-dir public/pkg --profiling --no-pack
# TODO: optimised web builds?

run_native_debug:
	cargo build --bin native_engine
	RUST_BACKTRACE=1 time cargo run --bin native_engine

run_native:
	cargo build --bin native_engine  --release
	time cargo run --bin native_engine  --release

run_web: build_web
	python test_web_server.py


# WARNING: DEPENDING ON HOW DETAILED THE INSTRUMENTATION IS, THIS WILL NEED MULTIPLE GIGABYTES OF RAM !!!
run_native_profiling:
	./run_profiling_session.sh

run_native_heap_profiling:
	cargo run --bin native_engine --release --features dhat-heap

clean:
	cargo clean
	rm './Cargo.lock'
	rm -r './public/pkg'