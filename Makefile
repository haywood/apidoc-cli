build:
	cargo build

test:
	cargo test
	./sh-test.sh

install: build
	cp target/debug/apidoc-cli /usr/local/bin/apidoc

clean:
	cargo clean
