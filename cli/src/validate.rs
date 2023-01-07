use std::{char, str};
use std::ffi::OsStr;
use std::path::Path;
use std::collections::{HashSet, HashMap};

pub struct Options<'a> {
	pub file: &'a OsStr,
	pub verbose: bool,
}

pub fn run(opts: &Options) {
	if opts.verbose {
		eprint!("opening {:?}... ", opts.file);
	}
	let file = udf::FileIO::open(opts.file).expect("error opening file");
	if opts.verbose {
		eprintln!("ok");
	}

	let mut validator = Validator::new(file);
	validator.run(opts);
}

struct Chain<'a> {
	parent: Option<&'a Chain<'a>>,
	dataset: udf::DatasetRef<'a>,
}

pub struct Validator {
	file: udf::FileIO,
	set: HashSet<udf::format::FileOffset>,
	datasets: usize,
	warns: usize,
	errors: usize,
}

impl Validator {
	pub fn new(file: udf::FileIO) -> Validator {
		Validator { file, set: HashSet::new(), datasets: 0, warns: 0, errors: 0 }
	}

	pub fn run(&mut self, opts: &Options) {
		let root_fo = self.file.root();
		self.run_rec(opts, root_fo, None);

		println!("Processed {} datasets", self.datasets);
		if self.warns == 0 && self.errors == 0 {
			println!("No warnings or errors found!");
		}
		else if self.errors == 0 {
			println!("Found {} warnings, but no errors!", self.warns);
		}
		else {
			println!("Found {} warnings, {} errors!", self.warns, self.errors);
		}
	}

	fn run_rec(&mut self, opts: &Options, fo: udf::format::FileOffset, parent: Option<&Chain<'_>>) {
		// Null datasets are not necessary an error, may happen due to incremental writing
		if fo.is_null() {
			self.warns += 1;
			eprintln!("warn: null dataset {:#x}:{:#x}", fo.offset, fo.size);
			return;
		}

		self.datasets += 1;

		// Datasets must be properly aligned.
		if !fo.is_aligned() {
			self.errors += 1;
			eprintln!("err: unaligned dataset {:#x}:{:#x}", fo.offset, fo.size);
			return;
		}

		// Check for big datasets
		if fo.size >= 0x100000000 {
			self.errors += 1;
			eprintln!("err: large dataset {:#x}:{:#x}", fo.offset, fo.size);
			return;
		}
		// Warn if dataset is larger than 1 GiB
		if fo.size > 0x40000000 {
			self.warns += 1;
			eprintln!("warn: large dataset {:#x}:{:#x}", fo.offset, fo.size);
		}

		// Detect cyclic datasets
		if !self.set.insert(fo) {
			eprintln!("cyclic dataset {:#x}:{:#x}", fo.offset, fo.size);
			return;
		}

		// Finally read the dataset
		if opts.verbose {
			eprint!("reading dataset {:#x}:{:#x}... ", fo.offset, fo.size);
		}
		let dataset = match self.file.read_dataset(fo) {
			Ok(dataset) => dataset,
			Err(err) => {
				self.errors += 1;
				eprintln!("{}", err);
				return;
			},
		};
		let dataset = dataset.as_ref();
		if opts.verbose {
			eprintln!("ok");
		}

		let lines = self.warns + self.errors;

		if dataset.tables.is_empty() {
			self.warns += 1;
			eprintln!("warn: empty dataset {:#x}:{:#x}", fo.offset, fo.size);
			return;
		}

		// if dataset.tables.is_sorted_by_key(|table| table.key_name) {
		// 	self.errors += 1;
		// 	eprintln!("err: dataset tables not sorted by key_name!");
		// 	return;
		// }

		let names = &dataset.names;

		let mut unique_names = HashMap::new();

		for (index, table) in dataset.tables.iter().enumerate() {
			if table.key_name == 0 {
				self.errors += 1;
				eprintln!("err: table (index={}) has null key_name!", index);
			}
			else {
				if let Some(other_index) = unique_names.insert(table.key_name, index) {
					self.errors += 1;
					eprintln!("err: table (index={} key_name={:#x}) with the same name already exists at index={}", index, table.key_name, other_index);
				}
			}

			let key_name = udf::NameOrHash(names.lookup(table.key_name));
			if key_name.0.is_err() {
				self.errors += 1;
				eprintln!("err: table {} invalid name!", key_name);
			}

			self.validate_shape(&key_name, table);

			let data = self.validate_data(&dataset, &key_name, table);

			if table.index_name != 0 {
				let index_name = udf::NameOrHash(names.lookup(table.index_name));
				if index_name.0.is_err() {
					self.errors += 1;
					eprintln!("err: table {} index_name={} not found", key_name, index_name);
				}
				if table.key_name == table.index_name {
					self.errors += 1;
					eprintln!("err: index into itself");
				}
			}

			// Validate the related table
			if table.related_name != 0 {
				self.validate_related(&dataset, &names, table);
			}
		}

		if self.warns + self.errors != lines {
			eprintln!();
		}

		// Recursively walk file offset datatables
		let chain = Chain { parent, dataset };
		for table in dataset.tables {
			if table.type_info == udf::format::T_FILE_OFFSET {
				if let Some(data) = dataset.get_data_ref(table) {
					let file_offsets = data.as_slice::<udf::format::FileOffset>().unwrap();
					for &fo in file_offsets {
						self.run_rec(opts, fo, Some(&chain));
					}
				}
			}
		}
	}

	fn validate_data<'a>(&mut self, dataset: &udf::DatasetRef<'a>, key_name: &udf::NameOrHash, table: &udf::format::TableDesc) -> Option<udf::DataRef<'a>> {
		dataset.get_data_ref(table)
	}

	// Check that the shape matches the data size
	fn validate_shape(&mut self, key_name: &udf::NameOrHash, table: &udf::format::TableDesc) {
		let prim_type = table.type_info & udf::format::TYPE_PRIM_MASK;
		let shape = udf::Shape::from_shape(table.type_info, table.data_shape);
		let data_size = table.data_size as usize;
		let shape_len = shape.len();

		let success = match prim_type {
			// Cannot verify the relationship between data size and shape
			udf::format::TYPE_PRIM_CUSTOM => {
				return;
			},
			udf::format::TYPE_PRIM_I8 | udf::format::TYPE_PRIM_U8 => {
				shape_len == data_size
			},
			udf::format::TYPE_PRIM_I16 | udf::format::TYPE_PRIM_U16 => {
				data_size % 2 == 0 && shape_len == data_size / 2
			},
			udf::format::TYPE_PRIM_I32 | udf::format::TYPE_PRIM_U32 | udf::format::TYPE_PRIM_F32 => {
				data_size % 4 == 0 && shape_len == data_size / 4
			},
			udf::format::TYPE_PRIM_I64 | udf::format::TYPE_PRIM_U64 | udf::format::TYPE_PRIM_F64 => {
				data_size % 8 == 0 && shape_len == data_size / 8
			},
			_ => {
				self.warns += 1;
				eprintln!("warn: table {} unknown primitive type {:#x}!", key_name, prim_type);
				return;
			},
		};

		if !success {
			self.errors += 1;
			let prim_name = udf::PrintTypeInfo::prim(prim_type).unwrap_or("?");
			eprintln!("err: table {} has shape {} with {} elements of {} but data size {:#x} does not match!",
				key_name, shape, shape_len, prim_name, data_size);
		}
	}

	fn validate_hint_none(&mut self, dataset: &udf::DatasetRef, key_name: &udf::NameOrHash, table: &udf::format::TableDesc) {

	}

	fn validate_hint_text(&mut self, dataset: &udf::DatasetRef, key_name: &udf::NameOrHash, table: &udf::format::TableDesc) {
		let prim_type = table.type_info & udf::format::TYPE_PRIM_MASK;
		let data_ref = match dataset.get_data_ref(table) {
			Some(data) => data,
			None => {
				self.errors += 1;
				eprintln!("err: table {} invalid data!", key_name);
				return;
			},
		};
		match prim_type {
			udf::format::TYPE_PRIM_I8 | udf::format::TYPE_PRIM_U8 => {
				if let Err(err) = std::str::from_utf8(data_ref.as_slice::<u8>().unwrap()) {
					self.errors += 1;
					eprintln!("err: table {} invalid utf8: {}", key_name, err);
				}
			},
			udf::format::TYPE_PRIM_U16 => {
				let words = data_ref.as_slice::<u16>().unwrap_or(&[]);
				if !char::decode_utf16(words.iter().copied()).all(|x| x.is_ok()) {
					self.errors += 1;
					eprintln!("err: table {} invalid utf16!", key_name);
				}
			},
			_ => {
				self.errors += 1;
				eprintln!("err: table {} is TYPE_HINT_TEXT but incompatible prim_type: {}", key_name, udf::PrintTypeInfo::prim(table.type_info).unwrap_or("?"));
			},
		}
	}

	fn validate_related(&mut self, dataset: &udf::DatasetRef<'_>, names: &udf::NamesRef<'_>, table: &udf::format::TableDesc) {
		let key_name = udf::NameOrHash(names.lookup(table.key_name));

		// Name must exist
		let related_name = udf::NameOrHash(names.lookup(table.related_name));
		if related_name.0.is_err() {
			self.errors += 1;
			eprintln!("err: table {} related {} name not found!", key_name, related_name);
		}

		// Should not be related to itself
		if table.key_name == table.related_name {
			self.warns += 1;
			eprintln!("warn: table {} related to itself", key_name);
			return;
		}

		// Find the related table
		let related_table = match dataset.find_table(table.related_name) {
			Some(related_table) => related_table,
			None => {
				self.errors += 1;
				eprintln!("err: table {} related table {} not found!", key_name, related_name);
				return;
			},
		};

		// Related tables must have the same shape
		let shape = udf::Shape::from_type_info(table.type_info, table.data_shape);
		let related_shape = udf::Shape::from_type_info(related_table.type_info, related_table.data_shape);

		if shape != related_shape {
			self.errors += 1;
			eprintln!("err: related tables {} ~ {} do not have the same shape, {} != {}", key_name, related_name, shape, related_shape);
		}
	}
}
