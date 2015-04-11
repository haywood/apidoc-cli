export PATH := $(PWD)/bin:$(PATH)

triples = x86_64-apple-darwin x86_64-unknown-linux-gnu
configure_args = $(foreach triple,$(triples),--target=$(triple))
nproc = $(shell nproc)
tests = $(foreach path, $(wildcard sh-test/*), $(notdir $(path)))

define make-release-target
$(triple): target/$(triple)/release/apidoc

target/$(triple)/release/apidoc: | rustc .rust/lib/$(triple)
	cargo build --verbose --release --target $(triple)

.rust/lib/$(triple): | rustc .rust/lib
	triple=$(triple) bash install-triple.sh
endef

define make-test-target
$(test):
	cd sh-test/$(test) && PS4='($$$${BASH_SOURCE}:$$$${LINENO}): ' bash -ex run.sh 2>&1 | diff expected.txt -
endef

$(foreach triple, $(triples), $(eval $(call make-release-target, triple)))
$(foreach test, $(tests), $(eval $(call make-test-target, test)))

build: rustc
	cargo build

test: build $(tests)
	@echo $(tests)

rustc: /usr/local/bin/rustc

/usr/local/bin/rustc:
	bash install-rust.sh

clean:
	cargo clean

.rust/lib:
	mkdir -p .rust/lib

release: $(triples)
