use udf::AsDataRef;

fn main() {
	let mut rng = urandom::new();

	// Generate random shape
	let x = rng.range(3..9); // Columns
	let y = rng.range(3..9); // Rows
	let z = rng.range(2..6); // Depth

	// Fill with random data
	let mut data = vec![0.0; x * y * z];
	for entry in &mut data {
		*entry = rng.range(-50..150) as f64 / 10.0;
	}

	// Print the data with different dimensions
	let data = data.as_data_ref();
	let mut ndprint = data.print().unwrap();

	ndprint.set_shape(udf::Shape::D3(x as u32, y as u32, z as u8));
	println!("\n{}", ndprint);
	println!("\n----------------\n");

	ndprint.set_shape(udf::Shape::D2(x as u32, (y * z) as u32));
	println!("{}", ndprint);
	println!("\n----------------\n");

	ndprint.set_shape(udf::Shape::D1((x * y * z) as u32));
	println!("{}", ndprint);
}
