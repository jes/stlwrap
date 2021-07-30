all: stlwrap

stlwrap:
	rustc stlwrap.rs

clean:
	rm -f stlwrap
