use std::{fs, str};
use std::io::{self, Read, Seek};
use std::ffi::OsStr;
use std::path::Path;

use dataview::PodMethods;

pub struct Options<'a> {
	pub file: &'a OsStr,
	pub file_offset: Option<udf::format::FileOffset>,
	pub path: &'a str,
	pub import: &'a OsStr,
	pub create_new: bool,
	pub verbose: bool,
}

pub fn run(opts: &Options) {
	let mut file = if opts.create_new {
		udf::FileIO::create(opts.file, [0; 4])
	}
	else {
		udf::FileIO::edit(opts.file)
	}.expect("open error");

	let ds_path = Path::new(opts.import).join("Dataset.ini");
	let ds_ini = fs::read_to_string(ds_path).expect("error opening Dataset.ini");

	let mut ds = parse_ini(opts, &ds_ini);

	let fo = file.add_dataset(&ds.finalize()).unwrap();
	file.set_root(fo);
	file.write_header().unwrap();
}

#[derive(Debug, Default)]
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
	let mut ds = udf::Dataset::new();
	for name in names.split(",") {
		ds.names.add(name, udf::hash(name));
	}

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

fn load_table(opts: &Options, ds: &mut udf::Dataset, desc: &IniTableDesc) {
	println!("{:#?}", desc);

	let udf::PrintTypeInfo(type_info) = desc.type_info.expect("Missing TypeInfo").parse().unwrap();
	let shape: udf::Shape = desc.shape.expect("Missing Shape").parse().unwrap();

	let key_name = udf::hash(desc.key_name);
	let index_name = desc.index.map(|index| udf::hash(index)).unwrap_or(0);
	let related_name = desc.related.map(|related| udf::hash(related)).unwrap_or(0);
	let file_name = desc.file_name.expect("Missing FileName");

	let mut byte_storage: Vec<u8>;
	let mut word_storage: Vec<u16>;
	let mut dword_storage: Vec<u32>;
	let mut qword_storage: Vec<u64>;

	let mut compressed: Vec<u8>;
	let mut compress_info = 0;

	let prim_type = type_info & udf::format::TYPE_PRIM_MASK;
	let storage = match prim_type {
		udf::format::TYPE_PRIM_CUSTOM => {
			let colon = file_name.find(":").unwrap();
			let file_type = &file_name[..colon];
			let file_name = &file_name[colon + 1..];
			let file_path = Path::new(opts.import).join(file_name);

			byte_storage = Vec::new();

			match file_type {
				"raw" => {
					let mut file = fs::File::open(&file_path).unwrap();
					file.read_to_end(&mut byte_storage).unwrap();
				},
				"npy" => {
					let mut file = fs::File::open(&file_path).unwrap();
					file.seek(io::SeekFrom::Start(0x80)).unwrap();
					file.read_to_end(&mut byte_storage).unwrap();
				},
				_ => unimplemented!("{}", file_type),
			}
			&byte_storage[..]
		},
		udf::format::TYPE_PRIM_I8 | udf::format::TYPE_PRIM_U8 => {
			byte_storage = vec![0u8; shape.len()];
			load_data(opts, file_name, &mut byte_storage);
			&byte_storage[..]
		},
		udf::format::TYPE_PRIM_I16 | udf::format::TYPE_PRIM_U16 => {
			word_storage = vec![0u16; shape.len()];
			load_data(opts, file_name, word_storage.as_bytes_mut());
			word_storage.as_bytes()
		},
		udf::format::TYPE_PRIM_F32 => {
			dword_storage = vec![0u32; shape.len()];
			load_data(opts, file_name, dword_storage.as_bytes_mut());
			compressed = Vec::new();
			// let mut stats = udf::compress::Stats::default();
			udf::compress::SimpleF32 { unit: 0.001 }.compress(&mut compressed, unsafe { std::mem::transmute(&dword_storage[..]) });
			// println!("Compressed: {:#?}", stats);
			compress_info = udf::format::COMPRESS_SIMPLE_F32;
			compressed.as_slice()
		},
		udf::format::TYPE_PRIM_I32 | udf::format::TYPE_PRIM_U32 => {
			dword_storage = vec![0u32; shape.len()];
			load_data(opts, file_name, dword_storage.as_bytes_mut());
			compressed = Vec::new();
			udf::compress::SimpleU32.compress(&mut compressed, &dword_storage);
			compress_info = udf::format::COMPRESS_SIMPLE_U32;
			compressed.as_slice()
		},
		udf::format::TYPE_PRIM_I64 | udf::format::TYPE_PRIM_U64 | udf::format::TYPE_PRIM_F64 => {
			qword_storage = vec![0u64; shape.len()];
			load_data(opts, file_name, qword_storage.as_bytes_mut());
			qword_storage.as_bytes()
		},
		prim => unimplemented!("{}", prim),
	};

	let data = udf::DataRef {
		bytes: storage,
		compress_info,
		shape: shape.encode().1,
		type_info,
	};

	ds.add_table(udf::TableRef { key_name, data, index_name, related_name });
}

fn load_data(opts: &Options, file_name: &str, data: &mut [u8]) {
	let colon = file_name.find(":").unwrap();
	let file_type = &file_name[..colon];
	let file_name = &file_name[colon + 1..];
	let file_path = Path::new(opts.import).join(file_name);

	match file_type {
		"raw" => {
			let mut file = fs::File::open(&file_path).unwrap();
			file.read_exact(data).unwrap();
		},
		"npy" => {
			let mut file = fs::File::open(&file_path).unwrap();
			file.seek(io::SeekFrom::Start(0x80)).unwrap();
			file.read_exact(data).unwrap();
		},
		_ => unimplemented!("{}", file_type),
	}
}
