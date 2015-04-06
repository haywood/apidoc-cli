build:
	cargo build

test: build
	./sh-test.sh

release:
	cargo build --release

install: build
	cp target/debug/apidoc-cli /usr/local/bin/apidoc

clean:
	cargo clean
