#!/bin/bash
set -e

cargo build --release
VERSION=$(cargo pkgid | sed 's/.*#//')
BINARY=ssh-manager-rust

cp target/release/$BINARY ${BINARY}-${VERSION}
echo "Linux release ${BINARY}-${VERSION}"

cargo build --target aarch64-linux-android --release
cp target/aarch64-linux-android/release/$BINARY ${BINARY}-${VERSION}-android
echo "Android release ${BINARY}-${VERSION}"