use udf::AsDataRef;



fn main() {
	// let data = [100, 100, 0, 0, 0, 1, 1, 2, 2, 2, 2, 2, 3, 3, 13, 4, 5, 5, 5, 5, 8191, 0xa0b0c0d, 0xa0908070];

	let mut rng = urandom::new();

	// for i in 0..100 {
	// 	println!("{}: {}", i, udf::compress::hash32(i) % 64);
	// }

	for _ in 0..10 {
		let max_bits = rng.range(2usize..20usize);
		let len = rng.range(10..100);
		let mut data = Vec::with_capacity(len);
		for _ in 0..len {
			let n_bits = rng.range(0..max_bits);
			data.push(rng.next::<u32>() & ((1 << n_bits) - 1));
		}

		let input_size = dataview::bytes(data.as_slice()).len();

		println!("\n\n# Testing compression on {} u32", data.len());
		println!("\n{}", data.as_data_ref().print().unwrap());

		let mut stream = Vec::new();
		udf::compress::SimpleU32.compress(&mut stream, &data);
		let compress_size = stream.len();

		println!("\n{}\n", udf::PrintHex(&stream));

		let mut decomp = vec![0u32; data.len()];
		udf::compress::SimpleU32::decompress(&mut decomp, dataview::bytes(stream.as_slice()));
		assert_eq!(&decomp, &data);

		println!("Compression ratio: {} / {}: {:.1} %",
			udf::FileSize(input_size as u64), udf::FileSize(compress_size as u64),
			compress_size as f64 / input_size as f64 * 100.0);
	}
}
