CC = rustc

bin:
	mkdir -p bin

jll.rs: bin
	$(CC) -obin/jll src/jll.rs

install: jll.rs
	mkdir -p /usr/bin
	cp bin/jll /usr/bin
	mkdir -p /usr/include/jll
	cp -r std /usr/include/jll
