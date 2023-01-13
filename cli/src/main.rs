use std::fmt;
use std::path::Path;

#[macro_use]
mod macros;

fn main() {
	let app = clap::command!("udf")
		.subcommand(
			clap::Command::new("new")
				.about("Create an empty UDF file")
				.arg(clap::arg!(<file> "The UDF file").allow_invalid_utf8(true))
				.arg(clap::arg!(--id [id] "The identifier"))
		)
		.subcommand(
			clap::Command::new("validate")
				.about("Check the UDF file for errors")
				.arg(clap::arg!(<file> "The UDF file").allow_invalid_utf8(true))
				.arg(clap::arg!(--verbose "Verbose output"))
		).subcommand(
			clap::Command::new("print")
				.about("Print dataset or datatable information")
				.arg(clap::arg!(<file> "The UDF file").allow_invalid_utf8(true))
				.arg(clap::arg!([path] "Path to the dataset"))
				.arg(clap::arg!(-p --"print-array" "Print the array contents"))
				.arg(clap::arg!(-f --format [format] "Format option: one of hex, flat, array (default array)"))
				.arg(clap::arg!(--"line-width" [line_width] "Sets the line width for the purpose of inserting line breaks (default 75)"))
				.arg(clap::arg!(--"file-offset" [file_offset] "File offset to the root dataset"))
				.arg(clap::arg!(--verbose "Verbose output"))
		).subcommand(
			clap::Command::new("export")
				.about("Export a dataset or datatable")
				.arg(clap::arg!(<file> "The UDF file").allow_invalid_utf8(true))
				.arg(clap::arg!(<path> "Path to the dataset"))
				.arg(clap::arg!(<output> "Output path").allow_invalid_utf8(true))
				.arg(clap::arg!(-f --format [format] "Format option: one of raw, npy (default raw)"))
				.arg(clap::arg!(--"file-offset" [file_offset] "File offset to the root dataset"))
				.arg(clap::arg!(--verbose "Verbose output"))
		).subcommand(
			clap::Command::new("import")
				.about("Import a dataset")
				.after_help(import::AFTER_HELP)
				.arg(clap::arg!(<file> "The UDF file").allow_invalid_utf8(true))
				.arg(clap::arg!(<import> "Path to import file describing the dataset").allow_invalid_utf8(true))
				.arg(clap::arg!(--"create-new" "Create a new UDF file instead of updating an existing UDF file"))
				.arg(clap::arg!(--"set-root" "Set the imported dataset as the root dataset"))
				.arg(clap::arg!(--verbose "Verbose output"))
		).subcommand(
			clap::Command::new("set-root")
				.about("Set the root dataset")
				.after_help(set_root::AFTER_HELP)
				.arg(clap::arg!(<file> "The UDF file").allow_invalid_utf8(true))
				.arg(clap::arg!(<"file-offset"> "The file offset to assign"))
		).arg_required_else_help(true);

	let matches = app.get_matches();
	if let Some(matches) = matches.subcommand_matches("new") {
		let file = matches.value_of_os("file").unwrap();
		let file: &Path = file.as_ref();
		let id = matches.value_of("id");

		let udf::PrintId(id) = match id {
			Some(id) => expect!(id.parse(), "Error parsing "{id:?}),
			None => Default::default(),
		};

		let mut writer = expect!(udf::FileIO::create(file, id),
			"Create UDF file='"{file.display()}"' id='"{udf::PrintId(id)}"'");
		writer.write_header().unwrap();
	}
	else if let Some(matches) = matches.subcommand_matches("validate") {
		let file = matches.value_of_os("file").unwrap();
		let verbose = matches.is_present("verbose");

		let ref opts = validate::Options { file, verbose };
		validate::run(opts);
	}
	else if let Some(matches) = matches.subcommand_matches("print") {
		let file = matches.value_of_os("file").unwrap();
		let file_offset = value_of_t(matches, "file-offset");
		let path = matches.value_of("path").unwrap_or("");
		let verbose = matches.is_present("verbose");
		let print_array = matches.is_present("print-array");
		let line_width = matches.value_of_t::<u32>("line-width").unwrap_or(75);
		let format = matches.value_of_t::<print::Format>("format").unwrap_or_default();

		let ref opts = print::Options { file, file_offset, path, verbose, print_array, line_width, format };
		print::run(opts);
	}
	else if let Some(matches) = matches.subcommand_matches("export") {
		let file = matches.value_of_os("file").unwrap().as_ref();
		let file_offset = value_of_t::<udf::format::FileOffset>(matches, "file-offset");
		let path = matches.value_of("path").unwrap_or("");
		let output = matches.value_of_os("output").unwrap().as_ref();
		let format = match value_of_t::<export::Format>(matches, "format") {
			Some(format) => format,
			// Select a default format based on extension of the output
			None => match Path::new(output).extension() {
				Some(s) if s == "npy" => export::Format::Npy,
				_ => export::Format::Raw,
			},
		};
		let verbose = matches.is_present("verbose");

		let ref opts = export::Options { file, file_offset, path, output, format, verbose };
		export::run(opts);
	}
	else if let Some(matches) = matches.subcommand_matches("import") {
		let file = matches.value_of_os("file").unwrap().as_ref();
		let import = matches.value_of_os("import").unwrap().as_ref();
		let create_new = matches.is_present("create-new");
		let set_root = matches.is_present("set-root");
		let verbose = matches.is_present("verbose");

		let ref opts = import::Options { file, import, create_new, set_root, verbose };
		import::run(opts);
	}
	else if let Some(matches) = matches.subcommand_matches("set-root") {
		let file = matches.value_of_os("file").unwrap().as_ref();
		let file_offset = value_of_t::<udf::format::FileOffset>(matches, "file-offset").unwrap();

		let ref opts = set_root::Options { file, file_offset };
		set_root::run(opts);
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

mod validate;
mod print;
mod export;
mod import;
mod set_root;

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

#[repr(transparent)]
pub struct Fmt<F: Fn(&mut fmt::Formatter) -> fmt::Result>(pub F);
impl<F: Fn(&mut fmt::Formatter) -> fmt::Result> fmt::Display for Fmt<F> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		(self.0)(f)
	}
}
