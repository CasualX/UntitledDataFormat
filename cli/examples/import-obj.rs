// Converts an obj file to a udf file
// import-obj <input.obj> <output.udf>

use udf::AsDataRef;

fn main() {
	let matches = clap::command!("import-obj")
		.arg(clap::arg!(<input> "The input obj file").allow_invalid_utf8(true))
		.arg(clap::arg!(<output> "The output udf file").allow_invalid_utf8(true))
		.get_matches();

	let input_path = matches.value_of_os("input").expect("Missing input.obj path");
	let output_path = matches.value_of_os("output").expect("Missing output.udf path");

	// Parse the input.obj file
	let content = std::fs::read_to_string(input_path).expect("Unable to read input file");
	let object = parse_obj(&content);

	let mut udf = udf::FileIO::create(output_path, [0; 4]).expect("Unable to write output file");

	let mut ds = udf::Dataset::new();
	ds.header.id = *b"OBJ\0";
	for name in ["c", "gn", "g", "v", "vn", "vt", "fv", "fvt", "fvn"] {
		ds.names.add(name, udf::hash(name));
	}

	ds.add_table(udf::TableRef {
		key_name: udf::hash!("c"),
		data: object.comment.as_data_ref(),
		..Default::default()
	});
	if object.g.len() > 0 {
		ds.add_table(udf::TableRef {
			key_name: udf::hash!("gn"),
			data: udf::build_string_array_utf8(object.gn.iter().map(|s| s.as_str()), &mut Vec::new()),
			..Default::default()
		});
		ds.add_table(udf::TableRef {
			key_name: udf::hash!("g"),
			data: object.g.as_data_ref(),
			index_name: udf::hash!("v"),
			related_name: udf::hash!("gn"),
			..Default::default()
		});
	}

	ds.add_table(udf::TableRef {
		key_name: udf::hash!("v"),
		data: object.v.as_data_ref(),
		..Default::default()
	});
	if object.vt.len() > 0 {
		ds.add_table(udf::TableRef {
			key_name: udf::hash!("vt"),
			data: object.vt.as_data_ref(),
			..Default::default()
		});
	}
	if object.vn.len() > 0 {
		ds.add_table(udf::TableRef {
			key_name: udf::hash!("vn"),
			data: object.vn.as_data_ref(),
			..Default::default()
		});
	}
	ds.add_table(udf::TableRef {
		key_name: udf::hash!("fv"),
		data: object.fv.as_data_ref(),
		index_name: udf::hash!("v"),
		..Default::default()
	});
	if object.fvt.len() > 0 {
		ds.add_table(udf::TableRef {
			key_name: udf::hash!("fvt"),
			data: object.fvt.as_data_ref(),
			index_name: udf::hash!("vt"),
			related_name: udf::hash!("fv"),
			..Default::default()
		});
	}
	if object.fvn.len() > 0 {
		ds.add_table(udf::TableRef {
			key_name: udf::hash!("fvn"),
			data: object.fvn.as_data_ref(),
			index_name: udf::hash!("vn"),
			related_name: udf::hash!("fv"),
			..Default::default()
		});
	}
	let fo = udf.add_dataset(&ds.finalize()).unwrap();

	udf.set_id(*b"OBJ\0");
	udf.set_root(fo);
	udf.write_header().unwrap();
}

#[derive(Default)]
struct Object {
	comment: String,
	gn: Vec<String>,
	g: Vec<udf::format::RangeU32>,
	v: Vec<udf::format::Coord3F32>,
	vt: Vec<udf::format::Coord2F32>,
	vn: Vec<udf::format::Coord3F32>,
	fv: Vec<udf::format::Index3U32>,
	fvn: Vec<udf::format::Index3U32>,
	fvt: Vec<udf::format::Index3U32>,
}

// https://en.wikipedia.org/wiki/Wavefront_.obj_file
fn parse_obj(string: &str) -> Object {
	let mut object = Object::default();

	let mut group_start = None;

	for line in string.lines() {
		let ts = line.split_ascii_whitespace().collect::<Vec<_>>();
		if ts.len() == 0 {
			continue;
		}
		match ts[0] {
			"#" => {
				object.comment.push_str(line);
				object.comment.push_str("\n");
			},
			"v" => {
				assert_eq!(ts.len(), 4);
				let x = ts[1].parse::<f32>().unwrap();
				let y = ts[2].parse::<f32>().unwrap();
				let z = ts[3].parse::<f32>().unwrap();
				object.v.push(udf::format::Coord3F32 { x, y, z });
			},
			"g" => {
				if let Some(group_start) = group_start {
					object.g.push(udf::format::RangeU32 { start: group_start as u32, end: object.v.len() as u32 });
				}
				object.gn.push(line[2..].to_string());
				group_start = Some(object.v.len());
			},
			"vt" => {
				assert_eq!(ts.len(), 3);
				let u = ts[1].parse::<f32>().unwrap();
				let v = ts[2].parse::<f32>().unwrap();
				object.vt.push(udf::format::Coord2F32 { x: u, y: v });
			},
			"vn" => {
				assert_eq!(ts.len(), 4);
				let x = ts[1].parse::<f32>().unwrap();
				let y = ts[2].parse::<f32>().unwrap();
				let z = ts[3].parse::<f32>().unwrap();
				object.vn.push(udf::format::Coord3F32 { x, y, z });
			},
			"f" => {
				assert_eq!(ts.len(), 4);
				let mut fv = [0; 3];
				let mut fvn = [None; 3];
				let mut fvt = [None; 3];
				for i in 0..3 {
					let mut tz = ts[i + 1].split("/");
					fv[i] = process_index(tz.next().unwrap().parse::<i32>().unwrap(), object.v.len());
					if let Some(a) = tz.next() {
						if let Ok(fvtc) = a.parse::<i32>() {
							fvt[i] = Some(process_index(fvtc, object.vt.len()));
						}

						if let Some(b) = tz.next() {
							fvn[i] = Some(process_index(b.parse::<i32>().unwrap(), object.vn.len()));
						}
					}
				}

				object.fv.push(udf::format::Index3U32(fv[0], fv[1], fv[2]));

				match fvn {
					[Some(fvnx), Some(fvny), Some(fvnz)] => {
						object.fvn.push(udf::format::Index3U32(fvnx, fvny, fvnz));
					},
					[None, None, None] => {},
					_ => panic!("inconsistent face vertex normals"),
				}

				match fvt {
					[Some(fvtx), Some(fvty), Some(fvtz)] => {
						object.fvt.push(udf::format::Index3U32(fvtx, fvty, fvtz));
					},
					[None, None, None] => {},
					_ => panic!("inconsistent face vertex texture coordinates"),
				}
			},
			_ => (),
		}
	}

	if let Some(group_start) = group_start {
		object.g.push(udf::format::RangeU32 { start: group_start as u32, end: object.v.len() as u32 });
	}

	object
}

fn process_index(i: i32, len: usize) -> u32 {
	let j = if i > 0 {
		(i - 1) as u32
	}
	else if i < 0 {
		(len as i32 + i) as u32
	}
	else {
		panic!("zero index");
	};

	assert!((j as usize) < len, "out of bounds");
	return j;
}