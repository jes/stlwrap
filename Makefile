all: stlwrap

stlwrap: src/*.rs
	cargo build --release && cp target/release/stlwrap .

install:
	cp stlwrap /usr/bin/stlwrap

clean:
	rm -f stlwrap
