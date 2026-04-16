#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "$0")" && pwd)"

rm -rf "$script_dir/dist/samples"
mkdir -p "$script_dir/dist/samples"

cd "$script_dir/dist/samples"
cargo run --bin udf-cli new empty.udf
cargo run --example playground
cargo run --example randomjob
cargo run --example import-obj ../../../data/bunny.object bunny.udf
cargo run --example import-obj ../../../data/seahorse.object seahorse.udf
