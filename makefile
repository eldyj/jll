CC = rustc

bin:
	mkdir -p bin

jll.rs: bin
	$(CC) -obin/jll src/jll.rs
