#!/bin/bash -e

cd $(dirname $0)
ROOT=$(pwd -P)
# put built CLI on path
export PATH="$ROOT/bin:$PATH"
export PS4='(${BASH_SOURCE}:${LINENO}): '
cd sh-test
for example in *; do
  if [ -d $example ]; then
    pushd >/dev/null $example
      echo "Running example in $example"
      bash -ex run.sh 2>&1 | diff expected.txt -
    popd >/dev/null
  fi
done
