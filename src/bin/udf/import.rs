use std::{fs, str};
use std::io::{self, Write};
use std::ffi::OsStr;
use std::path::Path;

pub struct Options<'a> {
	pub file: &'a OsStr,
	pub file_offset: Option<udf::format::FileOffset>,
	pub path: &'a str,
	pub import: &'a OsStr,
	pub verbose: bool,
}

pub fn run(opts: &Options) {
	let mut file = udf::FileIO::edit(opts.file).expect("open error");

	let ds_path = Path::new(opts.import).join("Dataset.ini");
	let ds_ini = fs::read_to_string(ds_path).expect("error opening Dataset.ini");

	let ds = parse_ini(opts, &ds_ini);
}

#[derive(Default)]
struct IniTableDesc<'a> {
	key_name: &'a str,
	type_info: Option<&'a str>,
	shape: Option<&'a str>,
	file_name: Option<&'a str>,
	index: Option<&'a str>,
	related: Option<&'a str>,
}

fn parse_ini(opts: &Options, s: &str) -> udf::Dataset {
	use ini_core::*;
	let mut parser = Parser::new(s);

	let mut id = None;
	let mut names = None;

	let mut desc = IniTableDesc::default();

	while let Some(item) = parser.next() {
		match item {
			Item::Section(sect) => {
				desc.key_name = sect;
				break;
			},
			Item::Property(key, value) => {
				match key {
					"Id" => id = Some(value),
					"Names" => names = Some(value),
					key => panic!("Unknown key: {}", key),
				}
			},
			_ => (),
		}
	}

	let names = names.expect("missing names");
	let names = names.split(",").collect::<Vec<_>>();
	let mut ds = udf::Dataset::create(&names);

	if let Some(id) = id {
		let udf::PrintId(id) = id.parse().unwrap();
		ds.header.id = id;
	}

	while let Some(item) = parser.next() {
		match item {
			Item::Section(sect) => {
				load_table(opts, &mut ds, &desc);
				desc = IniTableDesc::default();
				desc.key_name = sect;
			},
			Item::Property(key, value) => {
				match key {
					"TypeInfo" => desc.type_info = Some(value),
					"Shape" => desc.shape = Some(value),
					"FileName" => desc.file_name = Some(value),
					"Index" => desc.index = Some(value),
					"Related" => desc.related = Some(value),
					key => panic!("Unknown key: {}", key),
				}
			},
			_ => (),
		}
	}

	load_table(opts, &mut ds, &desc);

	return ds;
}

fn load_table(opts: &Options, ds: &mut udf::Dataset, desc: &IniTableDesc, ) {
	let udf::PrintTypeInfo(type_info) = desc.type_info.expect("Missing TypeInfo").parse().unwrap();
	let shape: udf::Shape = desc.shape.expect("Missing Shape").parse().unwrap();

	unimplemented!()
}
