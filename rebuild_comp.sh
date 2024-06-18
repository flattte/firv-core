#!/usr/bin/env bash
set -xe
FIRVC_ROOT="path to firv rust compiler root directory"
cd $FIRVC_ROOT
pwd_=$(pwd)
# the target triple must be changed on different targets
if [ -f "$FIRVC_ROOT/build/aarch64-apple-darwin/llvm/llvm-finished-building" ]; then
  rm "$FIRVC_ROOT/build/aarch64-apple-darwin/llvm/llvm-finished-building"
fi
./x build
./x build library --target riscv64gc-unknown-none-elf
cd $pwd_
