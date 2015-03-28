#!/bin/bash -e

cd $(dirname $0)
ROOT=$(pwd -P)
LOG="$ROOT/sh-test.log"
export PATH="$ROOT/bin:$PATH"
cd sh-test
echo "log file is $LOG"
for file in **.sh; do
  echo "Running $file"
  bash -ex $file 2>> $LOG >> $LOG
done
