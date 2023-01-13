use std::collections::HashSet;
use std::{fs, str};
use std::io::{self, Read, Seek};
use std::path::Path;

use dataview::PodMethods;

pub const AFTER_HELP: &str = "\
	The import file is an INI file describing the dataset to be created.\n\
	For more information see the project's readme:\n\
	\n\
	https://github.com/CasualX/UntitledDataFormat/blob/master/cli/readme.md\n\
	\n\
	The final file offset of the Dataset is printed to stdout.\n\
";

pub struct Options<'a> {
	pub file: &'a Path,
	pub import: &'a Path,
	pub create_new: bool,
	pub set_root: bool,
	pub verbose: bool,
}

pub fn run(opts: &Options) {
	// Read the import file
	let import = expect!(
		fs::read_to_string(opts.import),
		"Read import file='"{opts.import.display()}"'");

	// Parse and process the import file
	let mut ds = parse(opts, &import);

	// Create or open the UDF file for editing
	let mut file = if opts.create_new {
		expect!(udf::FileIO::create(opts.file, [0; 4]),
			"Create UDF file='"{opts.file.display()}"'")
	}
	else {
		expect!(udf::FileIO::edit(opts.file),
			"Open UDF file='"{opts.file.display()}"'")
	};

	// Add the dataset to the UDF file
	let fo = expect!(
		file.add_dataset(&ds.finalize()),
		"Add dataset file='"{opts.file.display()}"'");

	// Communicate the fileoffset
	println!("{}", fo);

	// Optionally set the newly added dataset as the root
	if opts.set_root {
		file.set_root(fo);
		file.write_header().unwrap();
	}
}

// The data source type
#[derive(Debug)]
enum Source {
	// Fill data with zeroes
	Zero,
	// Source is a npy file
	Npy,
	// Source is binary data from the file
	Raw,
	// Parse source from utf8 text file
	Parse,
}
impl std::str::FromStr for Source {
	type Err = udf::ParseError;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"zero" => Ok(Source::Zero),
			"npy" => Ok(Source::Npy),
			"raw" => Ok(Source::Raw),
			"parse" => Ok(Source::Parse),
			_ => Err(udf::ParseError::InvalidFormat)
		}
	}
}

#[derive(Debug, Default)]
struct IniTableDesc<'a> {
	line: u32,
	key_name: &'a str,
	type_info: Option<&'a str>,
	shape: Option<&'a str>,
	source: Option<&'a str>,
	file_path: Option<&'a str>,
	index_name: Option<&'a str>,
	related_name: Option<&'a str>,
}

fn parse(opts: &Options, s: &str) -> udf::Dataset {
	use ini_core::*;
	let mut parser = Parser::new(s);

	// Parse the lines up to the first section
	let mut id = None;
	while let Some(item) = parser.next() {
		match item {
			Item::SectionEnd => {
				break;
			},
			Item::Property(key, Some(value)) => {
				match key {
					"Id" => id = Some(value),
					key => error!("Unknown key: "{key}"\nLine "{parser.line()}),
				}
			},
			Item::Error(_) | Item::Property(_, None) => {
				error!("Syntax error at line "{parser.line()});
			},
			_ => (),
		}
	}

	let mut ds = udf::Dataset::new();
	let mut names = HashSet::new();

	// Parse the dataset identifier
	if let Some(id) = id {
		let udf::PrintId(id) = id.parse().unwrap();
		ds.header.id = id;
	}

	// Parse each section as a datatable and add it to the dataset
	let mut desc = IniTableDesc::default();
	while let Some(item) = parser.next() {
		match item {
			Item::Section(sect) => {
				desc = IniTableDesc::default();
				desc.line = parser.line();
				desc.key_name = sect;
			},
			Item::SectionEnd => {
				load_table(opts, &mut names, &mut ds, &desc);
			},
			Item::Property(key, Some(value)) => {
				match key {
					"TypeInfo" => desc.type_info = Some(value),
					"Shape" => desc.shape = Some(value),
					"Source" => desc.source = Some(value),
					"FilePath" => desc.file_path = Some(value),
					"IndexName" => desc.index_name = Some(value),
					"RelatedName" => desc.related_name = Some(value),
					key => error!("Unknown key: "{key}"\nLine "{parser.line()}),
				}
			},
			Item::Error(_) | Item::Property(_, None) => {
				error!("Syntax error at line "{parser.line()});
			},
			_ => (),
		}
	}

	// Add all the relevant names to the names table
	for name in &names {
		ds.names.add(name, udf::hash(name));
	}

	return ds;
}


fn load_table(opts: &Options, names: &mut HashSet<String>, ds: &mut udf::Dataset, desc: &IniTableDesc) {
	if opts.verbose {
		eprintln!("Loading Datatable {}...", desc.key_name);
	}

	let type_info_s = expect!(desc.type_info,
		"Datatable "{desc.key_name}": Missing TypeInfo");

	let udf::PrintTypeInfo(type_info) = expect!(type_info_s.parse(),
		"Datatable "{desc.key_name}": Invalid TypeInfo");

	let shape = expect!(desc.shape,
		"Datatable "{desc.key_name}": Missing Shape");

	let shape = expect!(shape.parse::<udf::Shape>(),
		"Datatable "{desc.key_name}": Invalid Shape");

	let key_name = udf::hash(desc.key_name);
	let index_name = desc.index_name.map(|index| udf::hash(index)).unwrap_or(0);
	let related_name = desc.related_name.map(|related| udf::hash(related)).unwrap_or(0);

	names.insert(desc.key_name.to_string());
	if let Some(index_name) = desc.index_name {
		names.insert(index_name.to_string());
	}
	if let Some(related_name) = desc.related_name {
		names.insert(related_name.to_string());
	}

	let mut su8: Vec<u8>;
	let si8: Vec<i8>;
	let su16: Vec<u16>;
	let si16: Vec<i16>;
	let su32: Vec<u32>;
	let si32: Vec<i32>;
	let su64: Vec<u64>;
	let si64: Vec<i64>;
	let sf32: Vec<f32>;
	let sf64: Vec<f64>;

	let prim_type = type_info & udf::format::TYPE_PRIM_MASK;

	let source = expect!(desc.source,
		"Datatable "{desc.key_name}": Missing Source");

	let source = expect!(source.parse::<Source>(),
		"Datatable "{desc.key_name}": Invalid Source: must be one of zero, raw, npy, parse");

	// Make the file path absolute relative to the import file
	let file_buf;
	let file_path = if let Some(file_path) = desc.file_path {
		let mut file_path = Path::new(file_path);
		if file_path.is_relative() {
			file_buf = opts.import.parent().unwrap().join(file_path);
			file_path = &file_buf;
		}
		Some(file_path)
	}
	else {
		None
	};

	fn open_file(opts: &Options, key_name: &str, file_path: Option<&Path>) -> fs::File {
		let file_path = expect!(file_path,
			"Datatable "{key_name}": Missing FilePath");

		if opts.verbose {
			eprintln!("Reading from {}...", file_path.display());
		}

		expect!(fs::File::open(file_path),
			"Datatable "{key_name}": Invalid FilePath '"{file_path.display()}"'")
	}

	let bytes = match source {
		Source::Zero => {
			let len = shape.len();
			match prim_type {
				udf::format::TYPE_PRIM_U8 => { su8 = vec![0u8; len]; su8.as_bytes() },
				udf::format::TYPE_PRIM_I8 => { su8 = vec![0u8; len]; su8.as_bytes() },
				udf::format::TYPE_PRIM_U16 => { su16 = vec![0u16; len]; su16.as_bytes() },
				udf::format::TYPE_PRIM_I16 => { su16 = vec![0u16; len]; su16.as_bytes() },
				udf::format::TYPE_PRIM_U32 => { su32 = vec![0u32; len]; su32.as_bytes() },
				udf::format::TYPE_PRIM_I32 => { su32 = vec![0u32; len]; su32.as_bytes() },
				udf::format::TYPE_PRIM_U64 => { su64 = vec![0u64; len]; su64.as_bytes() },
				udf::format::TYPE_PRIM_I64 => { su64 = vec![0u64; len]; su64.as_bytes() },
				udf::format::TYPE_PRIM_F32 => { su32 = vec![0u32; len]; su32.as_bytes() },
				udf::format::TYPE_PRIM_F64 => { su64 = vec![0u64; len]; su64.as_bytes() },
				_ => error!("Datatable "{desc.key_name}": Source type 'zero' not compatible with "{type_info_s}),
			}
		},

		Source::Raw => {
			let mut file = open_file(opts, desc.key_name, file_path);
			su8 = Vec::new();
			file.read_to_end(&mut su8).unwrap();
			su8.as_bytes()
		},

		Source::Npy => {
			let mut file = open_file(opts, desc.key_name, file_path);
			file.seek(io::SeekFrom::Start(0x80)).unwrap();

			su8 = Vec::new();
			file.read_to_end(&mut su8).unwrap();
			su8.as_slice()
		},

		Source::Parse => {
			// Read text and squash unnecessary characters to spaces
			let mut text = String::new();
			{
				let mut file = open_file(opts, desc.key_name, file_path);
				file.read_to_string(&mut text).unwrap();
			}
			parse_num::preprocess(&mut text);

			// Parse all the numbers found
			match prim_type {
				udf::format::TYPE_PRIM_U8 => { su8 = parse_num::parse_all(&text); su8.as_bytes() },
				udf::format::TYPE_PRIM_I8 => { si8 = parse_num::parse_all(&text); si8.as_bytes() },
				udf::format::TYPE_PRIM_U16 => { su16 = parse_num::parse_all(&text); su16.as_bytes() },
				udf::format::TYPE_PRIM_I16 => { si16 = parse_num::parse_all(&text); si16.as_bytes() },
				udf::format::TYPE_PRIM_U32 => { su32 = parse_num::parse_all(&text); su32.as_bytes() },
				udf::format::TYPE_PRIM_I32 => { si32 = parse_num::parse_all(&text); si32.as_bytes() },
				udf::format::TYPE_PRIM_U64 => { su64 = parse_num::parse_all(&text); su64.as_bytes() },
				udf::format::TYPE_PRIM_I64 => { si64 = parse_num::parse_all(&text); si64.as_bytes() },
				udf::format::TYPE_PRIM_F32 => { sf32 = parse_num::parse_all(&text); sf32.as_bytes() },
				udf::format::TYPE_PRIM_F64 => { sf64 = parse_num::parse_all(&text); sf64.as_bytes() },
				_ => error!("Datatable "{desc.key_name}": Source type 'parse' not compatible with "{type_info_s}),
			}
		},
	};

	// The data starts out uncompressed
	let compress_info = udf::format::COMPRESS_NONE;
	let data = udf::DataRef { bytes, type_info, compress_info, shape };

	ds.add_table(udf::TableRef { key_name, data, index_name, related_name });

	if opts.verbose {
		eprintln!("done");
	}
}

mod parse_num;
