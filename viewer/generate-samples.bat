@echo off
pushd "%~dp0"
rmdir /s /q dist\samples
mkdir dist\samples
cd dist\samples
cargo run --bin udf-cli new empty.udf
cargo run --example playground
cargo run --example randomjob
cargo run --example import-obj ../../../data/bunny.object bunny.udf
cargo run --example import-obj ../../../data/seahorse.object seahorse.udf
popd
