#!/bin/bash -e

LOG="$(mktemp -t apidoc-cli-test).log"
cd $(dirname $0)
ROOT=$(pwd -P)
export PATH="$ROOT/bin:$PATH"
cd sh-test
echo "log file is $LOG"
for file in **.sh; do
  echo "Running $file"
  bash -ex $file 2>> $LOG >> $LOG
done
