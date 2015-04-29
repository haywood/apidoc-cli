#!/bin/bash -ex

ARCH=$(uname -m)
OS=$(echo $(uname) | tr [[:upper:]] [[:lower:]])

if [ "$ARCH" != "x86_64" ] ; then
  echo >&2 "Cannot release from unsupported architecture $ARCH."
  exit 1
fi

if [[ "$OS" =~ "linux" ]] ; then
  TRIPLE="$ARCH-unknown-linux-gnu"
elif [[ "$OS" =~ "darwin" ]] ; then
  TRIPLE="$ARCH-apple-darwin"
fi

echo "TRIPLE=${TRIPLE:?}"

make release

cp target/release/apidoc /tmp/apidoc-$TRIPLE
