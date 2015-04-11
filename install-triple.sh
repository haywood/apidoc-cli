set -x
pushd /tmp
name="rust-nightly-$triple"
tar_name="${name}.tar.gz"
curl -s -O "https://static.rust-lang.org/dist/$tar_name"
tar -x -z -f "$tar_name"
popd
lib_dir=".rust/lib/rustlib/"
mkdir -p $lib_dir
cp -R /tmp/$name/rustc/lib/rustlib/$triple "$lib_dir"
