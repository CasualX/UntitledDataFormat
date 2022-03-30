use udf::AsDataRef;


fn main() {
	let mut writer = udf::FileIO::create("hello.udf", [0; 4]).unwrap();

	let mut ds = udf::Dataset::create(&["Points", "Contours", "Attributes", "Metadata", "Slices", "Heights", "Parts", "PartID"]);
	ds.add_table(udf::TableRef {
		key_name: udf::hash("Points"),
		data: FLOATS.as_data_ref(),
		index_name: 0,
		related_name: 0,
	});

	println!("{:#?}", ds.get_names());

	let fo = writer.add_dataset(ds.as_ref()).unwrap();
	writer.set_root(fo);
	writer.write_header().unwrap();
}

static FLOATS: [f32; 6] = [0.0, 1.0, 2.0, 3.0, 3.141592, 42.0];
