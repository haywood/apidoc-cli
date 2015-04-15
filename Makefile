export PATH := $(PWD)/bin:$(PATH)
export PATH := $(PWD)/.rust/bin:$(PATH)
export LD_LIBRARY_PATH := $(PWD)/.rust/lib:$(LD_LIBRARY_PATH)
export DYLD_LIBRARY_PATH := $(PWD)/.rust/lib:$(DYLD_LIBRARY_PATH)

nproc = $(shell nproc)
tests = $(foreach path, $(wildcard sh-test/*), $(notdir $(path)))

define make-test-target
$(test):
	cd sh-test/$(test) && PS4='($$$${BASH_SOURCE}:$$$${LINENO}): ' bash -ex run.sh 2>&1 | diff expected.txt -
endef

build: rustc
	.rust/bin/cargo build --verbose

test: build $(tests)

rustc: .rust/bin/rustc

.rust/bin/rustc:
	bash install-rust.sh

clean:
	.rust/bin/cargo clean --verbose

release:
	cargo build --release --verbose

$(foreach test, $(tests), $(eval $(call make-test-target, test)))
