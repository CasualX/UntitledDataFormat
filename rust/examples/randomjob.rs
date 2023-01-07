use udf::AsDataRef;

fn main() {
	let mut udf = udf::FileIO::create("job.udf", *b"JOB0").unwrap();

	let mut rng = urandom::new();

	let n_slices = rng.range(400..800) as usize;

	let base_height = rng.range(-10.0f32..10.0f32);
	let slice_thickness = rng.range(0..4) as f32 * 0.05 + 0.1;

	let mut slices_fo = Vec::with_capacity(n_slices);
	let mut heights = Vec::with_capacity(n_slices);

	for slice_index in 0..n_slices {
		let mut ds = udf::Dataset::new();
		for &name in &["Points", "Contours", "Attributes", "Metadata", "PartIndex", "Height", "PartsInfo"] {
			ds.names.add(name, udf::hash(name));
		}

		let height = slice_index as f32 * slice_thickness + base_height;
		heights.push(height);

		let mut points = Vec::new();
		let mut contours = Vec::new();
		let mut attributes = Vec::new();
		let mut part_indices = Vec::new();

		let n_contours = rng.range(5..20) as usize;
		for _ in 0..n_contours {
			generate_contour(&mut rng, &mut points, &mut contours, &mut attributes, &mut part_indices);
		}

		ds.add_table(udf::TableRef {
			key_name: udf::hash!("Height"),
			data: height.as_data_ref(),
			..Default::default()
		});
		ds.add_table(udf::TableRef {
			key_name: udf::hash!("Points"),
			data: points.as_data_ref(),
			..Default::default()
		});
		ds.add_table(udf::TableRef {
			key_name: udf::hash!("Contours"),
			data: contours.as_data_ref(),
			index_name: udf::hash!("Points"),
			..Default::default()
		});
		ds.add_table(udf::TableRef {
			key_name: udf::hash!("Attributes"),
			data: attributes.as_data_ref(),
			related_name: udf::hash!("Contours"),
			index_name: udf::hash!("Metadata"),
			..Default::default()
		});
		let metadata = "[{}, {}]";
		ds.add_table(udf::TableRef {
			key_name: udf::hash!("Metadata"),
			data: udf::DataRef {
				bytes: metadata.as_bytes(),
				type_info: udf::format::TYPE_HINT_JSON | udf::format::TYPE_DIM_1D | udf::format::TYPE_PRIM_CUSTOM,
				compress_info: udf::format::COMPRESS_NONE,
				shape: [2, 0],
			},
			..Default::default()
		});
		ds.add_table(udf::TableRef {
			key_name: udf::hash!("PartIndex"),
			data: part_indices.as_data_ref(),
			related_name: udf::hash!("Contours"),
			index_name: udf::hash!("PartsInfo"),
			..Default::default()
		});
		ds.header.id = *b"VL2\0";

		let fo = udf.add_dataset(&ds.finalize()).unwrap();
		slices_fo.push(fo);
	}

	{
		let mut ds = udf::Dataset::new();
		for &name in &["PartsInfo", "Slices", "Heights"] {
			ds.names.add(name, udf::hash(name));
		}

		let parts_info = "[]";
		ds.add_table(udf::TableRef {
			key_name: udf::hash!("PartsInfo"),
			data: udf::DataRef {
				bytes: parts_info.as_bytes(),
				type_info: udf::format::TYPE_HINT_JSON | udf::format::TYPE_DIM_1D | udf::format::TYPE_PRIM_CUSTOM,
				compress_info: udf::format::COMPRESS_NONE,
				shape: [0, 0],
			},
			..Default::default()
		});
		ds.add_table(udf::TableRef {
			key_name: udf::hash!("Heights"),
			data: heights.as_data_ref(),
			related_name: udf::hash!("Slices"),
			..Default::default()
		});
		ds.add_table(udf::TableRef {
			key_name: udf::hash!("Slices"),
			data: slices_fo.as_data_ref(),
			..Default::default()
		});
		ds.header.id = *b"STCK";

		let root_fo = udf.add_dataset(&ds.finalize()).unwrap();
		udf.set_root(root_fo);
		udf.write_header().unwrap();
	}
}

fn generate_contour(
	rng: &mut urandom::Random<impl urandom::Rng>,
	points: &mut Vec<udf::format::Coord2F32>,
	contours: &mut Vec<udf::format::RangeU32>,
	attributes: &mut Vec<udf::format::IndexU32>,
	part_indices: &mut Vec<udf::format::IndexU32>,
) {

	let n_points = rng.range(50..200) as usize;
	let radius = 10.0 + (50.0 - 10.0) * (200 - n_points) as f32 / 150.0;
	let start_angle = rng.range(0.0f32..360.0);
	let cx = rng.range(-100.0f32..200.0f32);
	let cy = rng.range(-100.0f32..200.0f32);

	let contour_start = points.len();
	let contour_end = contour_start + n_points + 1;

	for i in 0..n_points {
		let a = start_angle + (i as f32 / n_points as f32) * 360.0;
		let (sin_a, cos_a) = a.to_radians().sin_cos();
		let x = cx + cos_a * radius;
		let y = cy + sin_a * radius;
		points.push(udf::format::Coord2F32 { x, y });
	}
	// Repeat first point
	points.push(points[contour_start]);

	let (start, end) = (contour_start as u32, contour_end as u32);
	contours.push(udf::format::RangeU32 { start, end });

	let attr = rng.range(0u32..5u32);
	attributes.push(udf::format::IndexU32(attr));

	let pi = rng.range(0u32..2u32);
	part_indices.push(udf::format::IndexU32(pi));
}
