use std::path::Path;

pub const AFTER_HELP: &str = "\
	This is a dangerous operation that can corrupt the UDF file.\n\
	Prefer the `--set-root` option when importing a dataset.\n\
	The old root dataset's file offset is printed.\n\
";

pub struct Options<'a> {
	pub file: &'a Path,
	pub file_offset: udf::format::FileOffset,
}

pub fn run(opts: &Options) {
	let mut file = expect!(
		udf::FileIO::edit(opts.file),
		"Open UDF file='"{opts.file.display()}"'");

	let old_root = file.root();
	println!("{}", old_root);
	file.set_root(opts.file_offset);
	file.write_header().unwrap();
}
