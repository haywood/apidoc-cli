set -x
curl -s -o /tmp/rustup.sh https://static.rust-lang.org/rustup.sh
sh /tmp/rustup.sh --yes --channel=nightly --disable-sudo --prefix=$PWD/.rust --save
