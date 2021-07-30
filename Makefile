all: stlwrap

stlwrap:
	cargo build --release && cp target/release/stlwrap .

clean:
	rm -f stlwrap
