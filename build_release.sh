#!/bin/bash
set -e

cargo build --release
VERSION=$(cargo pkgid | sed 's/.*#//')
BINARY=ssh-manager-rust

cp target/release/$BINARY target/release/${BINARY}-${VERSION}
echo "Release ${BINARY}-${VERSION}"
