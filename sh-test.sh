#!/bin/bash

cd $(dirname $0)
ROOT=$(pwd -P)
export PATH="$ROOT/bin:$PATH"
cd sh-test
for file in **.sh; do
  echo "Running $file"
  bash -ex $file
done
