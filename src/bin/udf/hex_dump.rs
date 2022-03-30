use std::fs;
use std::ffi::OsStr;
use std::io::{self, Read, Seek};

pub struct Options<'a> {
	pub file: &'a OsStr,
	pub file_offset: Option<udf::format::FileOffset>,
}

pub fn run(opts: &Options) {
	let udf = udf::FileIO::open(opts.file).expect("error opening file");
	let fo = opts.file_offset.unwrap_or_else(|| udf.root());
	drop(udf);

	let mut file = fs::File::open(opts.file).expect("error opening file");
	file.seek(io::SeekFrom::Start(fo.offset)).expect("seek error");

	let mut buf = vec![0u8; fo.size as usize];
	file.read_exact(&mut buf).expect("read error");

	let stdout = io::stdout();
	let _ = crate::hex_dump(&mut stdout.lock(), &buf);
}
