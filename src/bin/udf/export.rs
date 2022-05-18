use std::{fmt, fs};
use std::io::{self, Write};
use std::ffi::OsStr;

pub struct Options<'a> {
	pub file: &'a OsStr,
	pub file_offset: Option<udf::format::FileOffset>,
	pub path: &'a str,
	pub output: &'a OsStr,
	pub verbose: bool,
}

pub fn run(opts: &Options) {
	let mut file = udf::FileIO::open(opts.file).expect("open error");

	let mut fo = opts.file_offset.unwrap_or_else(|| file.root());
	let mut path = opts.path;

	loop {
		println!("{}", fo);
		let dataset = file.read_dataset(fo).unwrap();
		let dataset = dataset.as_ref();
		let names = dataset.get_names();

		if path.is_empty() {
			unimplemented!("Export whole dataset")
		}

		match udf::PathEl::parse(&mut path) {
			Ok(udf::PathEl::Dir { name, index }) => {
				let table = match names.find(name).and_then(|hash| dataset.find_table(hash)) {
					Some(a) => a,
					None => break eprintln!("Dataset does not have a table named {name:?}!"),
				};

				let data_ref = dataset.get_data_ref(table).unwrap();
				if data_ref.type_info & (udf::format::TYPE_HINT_MASK | udf::format::TYPE_PRIM_MASK) == udf::format::TYPE_HINT_DATASET | udf::format::TYPE_PRIM_U64 {
					let fos = data_ref.as_slice::<udf::format::FileOffset>().expect("dataset is malformed");

					fo = match fos.get(index as usize) {
						Some(&fo) => fo,
						None => break eprintln!(""),
					};
					continue;
				}
				else {
					break eprintln!("The path does not refer to a dataset table!");
				}
				#[allow(unreachable_code)]
				{ unreachable!() }
			},
			Ok(udf::PathEl::Name(name)) => {
				if !path.is_empty() {
					break eprintln!("The path is malformed");
				}

				let table = match names.find(name).and_then(|hash| dataset.find_table(hash)) {
					Some(a) => a,
					None => break eprintln!("Dataset does not have a table named {name:?}!"),
				};

				let data = dataset.get_data_ref(table).unwrap();

				break export_npy(&data, opts.output).unwrap();
			},
			Err(_err) => {
				break eprintln!("The path is malformed");
			},
		}
	}
}

fn export_npy(data: &udf::DataRef<'_>, output: &OsStr) -> io::Result<()> {
	// Decompress the data or fail
	let mut storage = Vec::new();
	let data = data.decompress(&mut storage);

	if data.compress_info != udf::format::COMPRESS_NONE {
		panic!("Decompression failed, cannot store compressed data");
	}

	// Figure out the descr and shape for the data array
	let (descr, shape) = match data.type_info & udf::format::TYPE_PRIM_MASK {
		udf::format::TYPE_PRIM_U8 => ("|u1", data.shape()),
		udf::format::TYPE_PRIM_I8 => ("|i1", data.shape()),
		udf::format::TYPE_PRIM_U16 => ("<u2", data.shape()),
		udf::format::TYPE_PRIM_I16 => ("<i2", data.shape()),
		udf::format::TYPE_PRIM_U32 => ("<u4", data.shape()),
		udf::format::TYPE_PRIM_I32 => ("<i4", data.shape()),
		udf::format::TYPE_PRIM_U64 => ("<u8", data.shape()),
		udf::format::TYPE_PRIM_I64 => ("<i8", data.shape()),
		udf::format::TYPE_PRIM_F32 => ("<f4", data.shape()),
		udf::format::TYPE_PRIM_F64 => ("<f8", data.shape()),
		// Fall back to dumping the array as bytes
		_ => ("|u1", udf::Shape::D1(data.bytes.len() as u32)),
	};

	// Formatter for numpy's shape
	struct ArrayProtocolShape(udf::Shape);
	impl fmt::Display for ArrayProtocolShape {
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			match &self.0 {
				udf::Shape::Scalar => f.write_str("()"),
				udf::Shape::D1(x) => write!(f, "({},)", x),
				udf::Shape::D2(x, y) => write!(f, "({}, {})", x, y),
				udf::Shape::D3(x, y, z) => write!(f, "({}, {}, {})", x, y, z),
			}
		}
	}

	// Format the header
	let mut header = format!("{{'descr': '{}', 'fortran_order': False, 'shape': {}, }}", descr, ArrayProtocolShape(shape));

	// Pad the header so the array data starts at 64-byte aligned offset
	let pad_len = ((10 + header.len()) / 64 + 1) * 64 - 10;
	while header.len() < pad_len {
		let chr = if header.len() + 1 == pad_len { "\n" } else { " " };
		header.push_str(chr);
	}

	// Format the magic bytes + header length
	let mut magic = *b"\x93NUMPY\x01\x00\x00\x00";
	magic[8] = (header.len() & 0xff) as u8;
	magic[9] = (header.len() >> 8 & 0xff) as u8;

	// Write the npy file
	let mut output = fs::File::create(output)?;
	output.write_all(&magic)?;
	output.write_all(header.as_bytes())?;
	output.write_all(data.bytes)?;
	Ok(())
}
