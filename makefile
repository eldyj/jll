CC = rustc

bin:
	mkdir -p bin

jll.rs: bin
	$(CC) -obin/jll src/jll.rs

include:
	mkdir -p /usr/include/jll

std: include
	cp -r std /usr/include/jll

install: jll.rs std
	mkdir -p /usr/bin
	cp bin/jll /usr/bin
