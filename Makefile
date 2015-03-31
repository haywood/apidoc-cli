triples = x86_64-apple-darwin x86_64-unknown-linux-gnu
configure_args = $(foreach triple,$(triples),--target=$(triple))
nproc = $(shell nproc)

build: rustc
	cargo build

test: build
	./sh-test.sh

rustc: /usr/local/bin/rustc

/usr/local/bin/rustc:
	bash install-rust.sh

clean:
	cargo clean

.rust/lib:
	mkdir -p .rust/lib

define make-release-target
$(triple): target/$(triple)/release/apidoc

target/$(triple)/release/apidoc: | rustc .rust/lib/$(triple)
	cargo build --release --target $(triple)

.rust/lib/$(triple): | rustc .rust/lib
	triple=$(triple) bash install-triple.sh
endef

$(foreach triple, $(triples), $(eval $(call make-release-target, triple)))

release: $(triples)
