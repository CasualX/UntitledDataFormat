use std::{io, str};
use std::ffi::OsStr;

pub enum Format {
	/// Print the array as a hex dump.
	HexDump,
	/// Print as a flattened array.
	FlatArray,
	/// Print as a proper multidimensional array.
	NdArray,
}
impl Default for Format {
	fn default() -> Self {
		Format::NdArray
	}
}
impl str::FromStr for Format {
	type Err = super::StringError;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"hex" => Ok(Format::HexDump),
			"flat" => Ok(Format::FlatArray),
			"array" => Ok(Format::NdArray),
			_ => Err(super::StringError::from("invalid format")),
		}
	}
}

pub struct Options<'a> {
	pub file: &'a OsStr,
	pub file_offset: Option<udf::format::FileOffset>,
	pub path: &'a str,
	pub verbose: bool,
	pub print_array: bool,
	pub line_width: u32,
	pub format: Format,
}

pub fn run(opts: &Options) {
	let mut file = udf::FileIO::open(opts.file).expect("open error");

	let ref fo = opts.file_offset.unwrap_or_else(|| file.root());

	walk(&mut file, opts, fo, opts.path, None);
}

fn walk(file: &mut udf::FileIO, opts: &Options, fo: &udf::format::FileOffset, mut path: &str, parent: Option<&udf::WalkRef<udf::DatasetRef>>) {
	if opts.verbose {
		eprint!("reading dataset {}... ", fo);
	}
	let dataset = file.read_dataset(*fo).unwrap();
	if opts.verbose {
		eprintln!("ok");
	}
	let dataset = dataset.as_ref();
	let names = &dataset.names;

	if path.is_empty() {
		if opts.verbose {
			eprintln!();
		}
		print_dataset(fo, names, &dataset);
		return;
	}

	if opts.verbose {
		eprintln!("path={path:?}");
	}

	match udf::PathEl::parse(&mut path) {
		Ok(udf::PathEl::Dir { name, index }) => {
			let table = match names.find(name).and_then(|hash| dataset.find_table(hash)) {
				Some(a) => a,
				None => return eprintln!("Dataset does not have a table named {name:?}!"),
			};

			let data_ref = dataset.get_data_ref(table).unwrap();
			if data_ref.type_info & (udf::format::TYPE_HINT_MASK | udf::format::TYPE_PRIM_MASK) == udf::format::TYPE_HINT_DATASET | udf::format::TYPE_PRIM_U64 {
				let fos = data_ref.as_slice::<udf::format::FileOffset>().expect("dataset is malformed");

				let fo = match fos.get(index as usize) {
					Some(fo) => fo,
					None => return eprintln!(""),
				};

				let chain = udf::WalkRef {
					parent,
					instance: &dataset,
				};
				walk(file, opts, fo, path, Some(&chain));
			}
			else {
				return eprintln!("The path does not refer to a dataset table!");
			}
		},
		Ok(udf::PathEl::Name(name)) => {
			if opts.verbose {
				eprintln!();
			}

			if !path.is_empty() {
				return eprintln!("The path is malformed");
			}

			let table = match names.find(name).and_then(|hash| dataset.find_table(hash)) {
				Some(a) => a,
				None => return eprintln!("Dataset does not have a table named {name:?}!"),
			};

			print_table_header(names, table);

			if opts.print_array {
				let data_ref = match dataset.get_data_ref(table) {
					Some(data_ref) => data_ref,
					None => return eprintln!("Error reading table data!"),
				};
				let mut storage = Vec::new();
				let data_ref = data_ref.decompress(&mut storage);

				let f = io::stdout();
				let mut f = f.lock();
				let f: &mut dyn io::Write = &mut f;

				match opts.format {
					Format::HexDump => {
						let _ = write!(f, "```\n");
						let _ = crate::hex_dump(f, data_ref.as_slice::<u8>().unwrap());
						let _ = writeln!(f, "\n```");
					},
					Format::FlatArray | Format::NdArray => {

						let type_hint = data_ref.type_info & udf::format::TYPE_HINT_MASK;
						if type_hint == udf::format::TYPE_HINT_TEXT || type_hint == udf::format::TYPE_HINT_JSON {
							let text = match str::from_utf8(data_ref.bytes) {
								Ok(text) => text,
								Err(err) => return eprintln!("Error reading table data: {}!", err),
							};

							println!("{}", text);
						}
						else {
							let mut ndprint = match data_ref.print() {
								Ok(ndprint) => ndprint,
								Err(_) => return eprintln!("Error printing table data!"),
							};
							ndprint.set_line_width(opts.line_width);
							if matches!(opts.format, Format::FlatArray) {
								ndprint.set_shape(data_ref.shape().flatten());
							}
							let _ = write!(f, "```\n{}\n```", ndprint);
						}
					},
				};
			}
		},
		Err(_err) => {
			return eprintln!("The path is malformed");
		},
	}
}

pub fn print_dataset(fo: &udf::format::FileOffset, names: &udf::NamesRef, ds: &udf::DatasetRef) {
	println!("# Dataset\n");
	println!("File offset: {:#x}:{:#x}", fo.offset, fo.size);
	println!("File size: {}", udf::FileSize(fo.size));
	println!("Header size: {}", udf::FileSize(ds.header.size as u64));
	println!("Identifier: {}", udf::PrintId(ds.header.id));
	if ds.header.checksum != 0 {
		println!("Checksum:  {:#x}", ds.header.checksum);
	}
	println!();
	for table in ds.tables {
		print_table_header(names, table);
	}
}

pub fn print_table_header(names: &udf::NamesRef, table: &udf::format::TableDesc) {
	if table.key_name == 0 {
		println!("## Names\n\n{:#?}\n", names);
		return;
	}

	let key_name = udf::NameOrHash(names.lookup(table.key_name));
	let shape = udf::Shape::from_shape(table.type_info, table.data_shape);
	println!("## {}", key_name);
	println!();
	println!("Type info: {}  ", udf::PrintTypeInfo(table.type_info));
	if table.compress_info != 0 {
		println!("Compress info: {}  ", table.compress_info);
		let prim_size_of = match table.type_info & udf::format::TYPE_PRIM_MASK {
			udf::format::TYPE_PRIM_I8 | udf::format::TYPE_PRIM_U8 => Some(1),
			udf::format::TYPE_PRIM_I16 | udf::format::TYPE_PRIM_U16 => Some(2),
			udf::format::TYPE_PRIM_I32 | udf::format::TYPE_PRIM_U32 | udf::format::TYPE_PRIM_F32 => Some(4),
			udf::format::TYPE_PRIM_I64 | udf::format::TYPE_PRIM_U64 | udf::format::TYPE_PRIM_F64 => Some(8),
			_ => None,
		};
		if let Some(prim_size_of) = prim_size_of {
			let size = shape.len() * prim_size_of;
			let ratio = table.data_size as f64 / size as f64;
			println!("Compress ratio: {:.1}%  ", ratio * 100.0);
		}
	}
	if table.mem_start <= table.mem_end {
		// Only print the memory size if it doesn't match the expected value for data_size
		let mem_size = table.mem_end - table.mem_start;
		if (table.data_size.wrapping_sub(1) / 8).wrapping_add(1) != mem_size {
			println!("Memory size: {}  ", udf::FileSize(mem_size as u64 * 8));
		}
	}
	println!("Data size: {}  ", udf::FileSize(table.data_size as u64));
	println!("Data shape: {}  ", shape);
	if table.index_name != 0 {
		println!("Index name: {}  ", udf::NameOrHash(names.lookup(table.index_name)));
	}
	if table.related_name != 0 {
		println!("Related name: {}  ", udf::NameOrHash(names.lookup(table.related_name)));
	}
	println!();
}
