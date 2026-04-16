
Generate the sample files:

```
cargo run --package udf-cli new empty.udf
cargo run --example playground
cargo run --example randomjob
cargo run --package udf-cli --example import-obj data/bunny.object bunny.udf
cargo run --package udf-cli --example import-obj data/seahorse.object seahorse.udf
```
