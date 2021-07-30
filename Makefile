all: stlwrap

stlwrap: src/*.rs
	cargo build --release && cp target/release/stlwrap .

clean:
	rm -f stlwrap
