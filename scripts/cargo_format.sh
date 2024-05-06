#!/bin/bash

set -e

CURRENT_DIR=$(echo $(cd $(dirname $0) && pwd))

PROJECT_ROOT="${CURRENT_DIR}/.."

cd $PROJECT_ROOT

if ! $(rustup component list | grep "clippy" | grep "installed" > /dev/null); then
  rustup component add clippy
fi

if ! $(rustup component list | grep "rustfmt" | grep "installed" > /dev/null); then
  rustup component add rustfmt
fi

echo "run: cargo clippy --no-deps --fix --allow-dirty"

cargo clippy --no-deps --fix --allow-dirty

echo "run: cargo fmt"

cargo fmt
