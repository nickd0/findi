#!/bin/bash

# NOTE: requires cross: https://github.com/rust-embedded/cross

set -e

docker ps >/dev/null || (echo "Docker not running!" && exit 1)

# TODO: pull suppported targets from cargo.toml?

TARGETS=$1
[ -n $TARGETS ] || TARGETS="arm-unknown-linux-gnueabihf x86_64-unknown-linux-gnu x86_64-apple-darwin"

dir=$PWD

for target in $TARGETS
do
  echo "Building $target..."
  if [ $target = "x86_64-apple-darwin" ];
  then
    cargo build --bin=findi --release
  else
    cross build --bin=findi --release --target $target
  fi

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

echo "Done building $TARGETS"
