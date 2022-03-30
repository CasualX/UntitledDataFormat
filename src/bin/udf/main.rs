
fn main() {

	let matches = clap::command!("udf")
		.subcommand(
			clap::Command::new("validate")
				.about("Checks the UDF file for errors")
				.arg(clap::arg!(<file> "The UDF file").allow_invalid_utf8(true))
				.arg(clap::arg!(-v --verbose "Verbose output"))
		).subcommand(
			clap::Command::new("print")
				.about("Prints info about a dataset")
				.arg(clap::arg!(<file> "The UDF file").allow_invalid_utf8(true))
				.arg(clap::arg!(--"file-offset" [file_offset] "File offset to the dataset"))
				.arg(clap::arg!([path] "Path to the dataset"))
				.arg(clap::arg!(-v --verbose "Verbose output"))
				.arg(clap::arg!(-p --"print-array" "Print the array contents"))
				.arg(clap::arg!(-f --format [format] "Format option, one of hex, flat, array (default array)"))
				.arg(clap::arg!(--"line-width" [line_width] "Sets the line width for the purpose of inserting line breaks (default 75)"))
		).subcommand(
			clap::Command::new("hex-dump")
				.about("Hex dump a dataset.")
				.arg(clap::arg!(<file> "The UDF file").allow_invalid_utf8(true))
				.arg(clap::arg!(--"file-offset" [file_offset] "Optional file offset to the dataset, if absent start fomr the root"))
		).get_matches();

	if let Some(matches) = matches.subcommand_matches("validate") {
		let udf_path = matches.value_of_os("file").unwrap();

		let opts = validate::Options {
			verbose: matches.is_present("verbose"),
		};

		if opts.verbose {
			eprint!("opening {:?}... ", udf_path);
		}
		let udf_file = udf::FileIO::open(udf_path).expect("error opening file");
		if opts.verbose {
			eprintln!("ok");
		}

		let mut validator = validate::Validator::new(udf_file, opts);
		validator.run();
	}
	else if let Some(matches) = matches.subcommand_matches("print") {
		let file = matches.value_of_os("file").unwrap();
		let file_offset = value_of_t(matches, "file-offset");
		let path = matches.value_of("path");
		let verbose = matches.is_present("verbose");
		let print_array = matches.is_present("print-array");
		let line_width = matches.value_of_t::<u32>("line-width").unwrap_or(75);
		let format = matches.value_of_t::<print::Format>("format").unwrap_or_default();

		let opts = print::Options { file, file_offset, path, verbose, print_array, line_width, format };
		print::run(&opts);
	}
	else if let Some(matches) = matches.subcommand_matches("hex-dump") {
		let file = matches.value_of_os("file").unwrap();
		let file_offset = value_of_t::<udf::format::FileOffset>(matches, "file-offset");
		let opts = hex_dump::Options { file, file_offset };
		hex_dump::run(&opts);
	}
	else {
		unreachable!()
	}
}

fn value_of_t<T>(matches: &clap::ArgMatches, name: &str) -> Option<T> where T: std::str::FromStr, T::Err: std::error::Error {
	match matches.value_of_t(name) {
		Ok(x) => Some(x),
		Err(err) if err.kind() == clap::ErrorKind::ArgumentNotFound => None,
		Err(err) => panic!("{}", err),
	}
}

mod error;
use self::error::StringError;

mod validate;
mod print;
mod hex_dump;

/*
Ideas:

Import/export JSON/CSV/binary
Syntax for walking dataset tables

*/

use std::io;
fn hex_dump(f: &mut dyn io::Write, bytes: &[u8]) -> io::Result<()> {
	write!(f, "  Offset 00 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F Decoded text")?;
	let mut offset = 0;
	while offset < bytes.len() {
		write!(f, "\n{:08x} ", offset)?;

		for i in 0..16 {
			if let Some(byte) = bytes.get(offset + i) {
				write!(f, "{:02x} ", byte)?;
			}
			else {
				write!(f, "   ")?;
			}
		}

		for i in 0..16 {
			if let Some(&byte) = bytes.get(offset + i) {
				let chr = if byte >= 0x20 && byte < 0x7f { byte as char } else { '.' };
				write!(f, "{}", chr)?;
			}
			else {
				break;
			}
		}

		offset += 16;
	}
	Ok(())
}
