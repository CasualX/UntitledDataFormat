use std::{fmt, io, mem, str};
use crate::*;

/// Binary search-based names lookup table.
///
/// Key names are hashed and refered to by their hash.
/// A dedicated names table translates these keys back into strings.
#[derive(Copy, Clone, Default)]
pub struct NamesRef<'a> {
	/// Slice of name descriptors.
	///
	/// The descriptors must be sorted ascending by their name hash.
	pub entries: &'a [format::LookupEntry],

	/// UTF-8 encoded slice of strings.
	///
	/// The name descriptors contain offsets into this slice.
	/// Each name string is nul terminated to aid C/C++ implementations (not included in returned strings).
	pub strings: &'a [u8],
}

impl<'a> NamesRef<'a> {
	/// Creates an empty instance.
	#[inline]
	pub const fn new() -> NamesRef<'a> {
		NamesRef { entries: &[], strings: &[] }
	}

	#[inline]
	pub(crate) fn from_data(data: &'a [u8], len: usize) -> Option<NamesRef<'a>> {
		let view = dataview::DataView::from(data);
		let names = view.try_slice::<format::LookupEntry>(0, len)?;
		let strings = data.get(mem::size_of_val(names)..)?;
		Some(NamesRef { entries: names, strings })
	}

	/// Creates an owned instance.
	#[inline]
	pub fn to_owned(&self) -> Names {
		Names {
			entries: self.entries.to_owned(),
			strings: self.strings.to_owned(),
		}
	}

	/// Lookup the name for a given hash.
	///
	/// Returns `None` if the hash wasn't found,
	/// there was an out of bounds error fetching the name string,
	/// or the string failed to convert to UTF-8.
	#[inline]
	pub fn lookup(&self, hash: u32) -> Result<&'a str, u32> {
		if hash == 0 {
			return Err(hash);
		}
		let index = match self.entries.binary_search_by_key(&hash, |name| name.hash) {
			Ok(index) => index,
			Err(_) => return Err(hash),
		};
		let desc = match self.entries.get(index) {
			Some(desc) => desc,
			None => return Err(hash),
		};
		let name = match name(self.strings, desc) {
			Some(name) => name,
			None => return Err(hash),
		};
		Ok(name)
	}

	/// Finds the hash for a given name string.
	#[inline]
	pub fn find(&self, name: &str) -> Option<u32> {
		for (hash, s) in self.iter() {
			if s == Some(name) {
				return Some(hash);
			}
		}
		return None;
	}

	/// Iterator over the name entries in this table.
	#[inline]
	pub fn iter(&self) -> impl 'a + Clone + Iterator<Item = (u32, Option<&'a str>)> {
		let &NamesRef { entries: names, strings } = self;
		names.iter().map(move |desc| (desc.hash, name(strings, desc)))
	}

	/// Iterator over the names in this table.
	#[inline]
	pub fn names(&self) -> impl 'a + Clone + Iterator<Item = Option<&'a str>> {
		let &NamesRef { entries: descs, strings } = self;
		descs.iter().map(move |desc| name(strings, desc))
	}

	/// Returns the file size in bytes.
	#[inline]
	pub fn file_size(&self) -> usize {
		mem::size_of_val(self.entries) + mem::size_of_val(self.strings)
	}

	pub fn write(&self, w: &mut dyn io::Write) -> io::Result<()> {
		w.write_all(dataview::bytes(self.entries))?;
		w.write_all(self.strings)?;
		Ok(())
	}
}

#[inline]
fn name<'a>(strings: &'a [u8], desc: &format::LookupEntry) -> Option<&'a str> {
	let start = desc.offset as usize;
	let end = start + desc.len as usize;
	let name = strings.get(start..end)?;
	str::from_utf8(name).ok()
}

struct NamesRefFormatter<'a>(NamesRef<'a>);

impl<'a> fmt::Debug for NamesRefFormatter<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let mut dm = f.debug_map();
		for (hash, name) in self.0.iter() {
			dm.entry(&format_args!("{:#010x}", hash), &format_args!("{:?}", name));
		}
		dm.finish()
	}
}
impl<'a> fmt::Debug for NamesRef<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_tuple("NamesRef")
			.field(&NamesRefFormatter(*self))
			.finish()
	}
}

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct NameOrHash<'a>(pub Result<&'a str, u32>);

impl<'a> fmt::Display for NameOrHash<'a> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self.0 {
			Ok(name) => write!(f, "{}", name),
			Err(hash) => write!(f, "{:#010x}", hash),
		}
	}
}
impl<'a> fmt::Debug for NameOrHash<'a> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self.0 {
			Ok(name) => write!(f, "{:?}", name),
			Err(hash) => write!(f, "{:#010x}", hash),
		}
	}
}
