use std::{fs, str};
use std::io::{self, Write};
use std::ffi::OsStr;
use std::path::Path;

#[derive(Copy, Clone, Debug)]
pub enum Format {
	/// Just write the data bytes to disk.
	Raw,
	/// Numpy file format.
	Npy,
}
impl Default for Format {
	fn default() -> Self {
		Format::Raw
	}
}
impl str::FromStr for Format {
	type Err = super::StringError;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"raw" => Ok(Format::Raw),
			"npy" => Ok(Format::Npy),
			_ => Err(super::StringError::from("invalid format")),
		}
	}
}

pub struct Options<'a> {
	pub file: &'a OsStr,
	pub file_offset: Option<udf::format::FileOffset>,
	pub path: &'a str,
	pub output: &'a OsStr,
	pub format: Format,
	pub verbose: bool,
}

pub fn run(opts: &Options) {
	let mut file = udf::FileIO::open(opts.file).expect("open error");

	let mut fo = opts.file_offset.unwrap_or_else(|| file.root());
	let mut path = opts.path;

	loop {
		if opts.verbose {
			println!("{}", fo);
		}
		let dataset = file.read_dataset(fo).unwrap();
		let dataset = dataset.as_ref();
		let names = dataset.get_names();

		if path.is_empty() {
			let output_dir = Path::new(opts.output);
			if let Err(err) = std::fs::create_dir(output_dir) {
				return eprintln!("Error creating output directory at {}: {}", output_dir.display(), err);
			}

			let mut ini = String::new();
			{
				use std::fmt::Write;
				let _ = write!(ini, "Id={}\n", udf::PrintId(dataset.header.id));
				let _ = write!(ini, "Names={}\n", crate::Fmt(move |f| {
					let mut comma = false;
					for name in names.names().filter_map(|name| name) {
						if comma {
							f.write_str(",")?;
						}
						comma = true;
						f.write_str(name)?;
					}
					Ok(())
				}));
			}

			for table in dataset.tables {
				if table.type_info == udf::format::TYPE_NAMES {
					continue;
				}

				let key_name = udf::NameOrHash(names.lookup(table.key_name));

				if let Err(err) = export_table(opts, &dataset, &names, table, Some(key_name), &mut ini) {
					eprintln!("Error exporting {}: {}", key_name, err);
					continue;
				}
			}

			// Write the Dataset.ini summary
			if let Err(err) = fs::write(output_dir.join("Dataset.ini"), ini) {
				eprintln!("Error writing Dataset.ini: {}", err);
			}

			println!("Exported {:?} to {}", opts.path, Path::new(opts.output).display());
			return;
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

				let mut desc = String::new();
				export_table(opts, &dataset, &names, table, None, &mut desc).unwrap();
				print!("{}", desc);
				break;
			},
			Err(_err) => {
				break eprintln!("The path is malformed");
			},
		}
	}
}

fn export_table(opts: &Options, ds: &udf::DatasetRef<'_>, names: &udf::NamesRef<'_>, table: &udf::format::TableDesc, name: Option<udf::NameOrHash>, desc: &mut String) -> io::Result<()> {
	let data = match ds.get_data_ref(table) {
		Some(data) => data,
		None => panic!("Unable to retrieve {:?}'s data", name),
	};

	// Decompress the data or fail
	let mut storage = Vec::new();
	let data = data.decompress(&mut storage);

	if data.compress_info != udf::format::COMPRESS_NONE {
		panic!("Decompression failed, cannot export compressed data");
	}

	let (path_buf, path);

	match opts.format {
		Format::Raw => {
			path = match name {
				Some(name) => {
					path_buf = Path::new(opts.output).join(name.to_string());
					&path_buf
				},
				None => {
					Path::new(opts.output)
				},
			};

			let mut fd = fs::File::create(path)?;
			fd.write_all(data.bytes)?;
		},
		Format::Npy => {
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
			let fmt_shape = crate::Fmt(|f| {
				match shape {
					udf::Shape::Scalar => f.write_str("()"),
					udf::Shape::D1(x) => write!(f, "({},)", x),
					udf::Shape::D2(x, y) => write!(f, "({}, {})", x, y),
					udf::Shape::D3(x, y, z) => write!(f, "({}, {}, {})", x, y, z),
				}
			});

			// Format the header
			let mut header = format!("{{'descr': '{}', 'fortran_order': False, 'shape': {}, }}", descr, fmt_shape);

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

			// Figure out the file name
			path = match name {
				Some(file_name) => {
					path_buf = Path::new(opts.output).join(format!("{}.npy", file_name));
					&path_buf
				},
				None => {
					Path::new(opts.output)
				},
			};

			// Write the npy file
			let mut fd = fs::File::create(path)?;
			fd.write_all(&magic)?;
			fd.write_all(header.as_bytes())?;
			fd.write_all(data.bytes)?;
		},
	}

	// Write descriptor
	{
		use std::fmt::Write;
		let _ = write!(desc, "\n[{}]\n", udf::NameOrHash(names.lookup(table.key_name)));
		let _ = write!(desc, "TypeInfo={}\n", udf::PrintTypeInfo(table.type_info));
		if table.compress_info != udf::format::COMPRESS_NONE {
			let _ = write!(desc, "CompressInfo={}\n", table.compress_info);
		}
		let _ = write!(desc, "Shape={}\n", udf::Shape::from_shape(table.type_info, table.data_shape));
		let _ = write!(desc, "FileName={}:{}\n",
			match opts.format { Format::Raw => "raw", Format::Npy => "npy" },
			path.file_name().unwrap().to_string_lossy());
		if table.index_name != 0 {
			let _ = write!(desc, "Index={}\n", udf::NameOrHash(names.lookup(table.index_name)));
		}
		if table.related_name != 0 {
			let _ = write!(desc, "Related={}\n", udf::NameOrHash(names.lookup(table.related_name)));
		}
	}

	Ok(())
}
