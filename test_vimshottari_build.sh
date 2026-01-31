#!/bin/bash
cd "$(dirname "$0")"
cargo build -p engine-vimshottari 2>&1
