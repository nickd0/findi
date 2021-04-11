#!/bin/bash

# NOTE: requires cross: https://github.com/rust-embedded/cross

set -e

docker ps >/dev/null || (echo "Docker not running!" && exit 1)

# TODO: pull suppported targets from cargo.toml?

TARGETS="arm-unknown-linux-gnueabihf x86_64-unknown-linux-gnu"

dir=$PWD

for target in $TARGETS
do
  echo "Building $target..."
  cross build --bin=findi --release --target $target

  if [ ! "$?" -eq "0" ];
  then
    echo "Build for $target failed!"
    exit 1
  fi

  cd "target/$target/release"
  tar czvf "$dir/release/$target.tar.gz" findi >/dev/null
  cd "$dir/release/"
  shasum -a 256 "$target.tar.gz" > "$target.sha256"
  cd $dir
done

echo "Building Darwin release"
cargo build --bin=findi --release
cd target/release

target="x86_64-apple-darwin"
tar czvf "$dir/release/$target.tar.gz" findi >/dev/null
cd "$dir/release/"
shasum -a 256 "$target.tar.gz" > "$target.sha256"
cd $dir

echo "Done"

