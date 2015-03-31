#!/bin/bash -e

cd $(dirname $0)
ROOT=$(pwd -P)
# put built CLI on path
export PATH="$ROOT/bin:$PATH"
cd sh-test
for example in *; do
  if [ -d $example ]; then
    pushd $example
      echo "Running example in $example"
      bash -ex run.sh 2>&1 | diff expected.txt -
    popd
  fi
done
