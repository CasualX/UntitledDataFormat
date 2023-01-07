
use dataview::PodMethods;

pub mod format;

mod fileio;
mod dataset_ref;
mod dataset;
mod table;
mod names_ref;
mod names;
mod shape;
mod data;
mod asdata;
mod hash;
mod file_offset;
mod print;
mod print_float;
mod path;
mod walk;
mod utils;
mod string_array;

pub use self::fileio::FileIO;
pub use self::dataset_ref::DatasetRef;
pub use self::dataset::Dataset;
pub use self::table::TableRef;
pub use self::names_ref::{NamesRef, NameOrHash};
pub use self::names::Names;
pub use self::shape::Shape;
pub use self::data::DataRef;
pub use self::asdata::AsDataRef;
pub use self::hash::hash;
pub use self::file_offset::ParseError;
pub use self::print::PrintArray;
use self::print_float::{PrintF32, PrintF64};
pub use self::path::PathEl;
pub use self::walk::WalkRef;
pub use self::utils::{PrintId, PrintTypeInfo, FileSize, PrintHex, Final};
pub use self::string_array::build_string_array_utf8;

pub mod compress;
