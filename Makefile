export PATH := $(PWD)/bin:$(PATH)
export PATH := $(PWD)/.rust/bin:$(PATH)
export LD_LIBRARY_PATH := $(PWD)/.rust/lib:$(LD_LIBRARY_PATH)
export DYLD_LIBRARY_PATH := $(PWD)/.rust/lib:$(DYLD_LIBRARY_PATH)

triples = x86_64-apple-darwin x86_64-unknown-linux-gnu
configure_args = $(foreach triple,$(triples),--target=$(triple))
nproc = $(shell nproc)
tests = $(foreach path, $(wildcard sh-test/*), $(notdir $(path)))

define make-release-target
$(triple): target/$(triple)/release/apidoc

target/$(triple)/release/apidoc: | rustc .rust/lib/rustlib/$(triple)
	.rust/bin/cargo build --verbose --release --target $(triple)

.rust/lib/rustlib/$(triple): | rustc
	triple=$(triple) bash install-triple.sh
endef

define make-test-target
$(test):
	cd sh-test/$(test) && PS4='($$$${BASH_SOURCE}:$$$${LINENO}): ' bash -ex run.sh 2>&1 | diff expected.txt -
endef

build: rustc
	.rust/bin/cargo build

test: build $(tests)

rustc: .rust/bin/rustc

.rust/bin/rustc:
	bash install-rust.sh

clean:
	.rust/bin/cargo clean

release: $(triples)

$(foreach triple, $(triples), $(eval $(call make-release-target, triple)))

$(foreach test, $(tests), $(eval $(call make-test-target, test)))
