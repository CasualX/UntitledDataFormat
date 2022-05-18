use std::{fmt, mem, slice, str};
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
	pub descs: &'a [format::NameDesc],

	/// UTF-8 encoded slice of strings.
	///
	/// The name descriptors contain offsets into this slice.
	/// Each name string is nul terminated to aid C/C++ implementations (not included in returned strings).
	pub names: &'a [u8],
}

impl<'a> NamesRef<'a> {
	/// Creates an empty instance.
	#[inline]
	pub const fn new() -> NamesRef<'a> {
		NamesRef { descs: &[], names: &[] }
	}

	#[inline]
	pub(crate) fn from_data(data: &'a [u8], len: usize) -> Option<NamesRef<'a>> {
		let descs = data.as_data_view().try_slice::<format::NameDesc>(0, len)?;
		let names = data.get(mem::size_of_val(descs)..)?;
		Some(NamesRef { descs, names })
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
		let index = match self.descs.binary_search_by_key(&hash, |desc| desc.hash) {
			Ok(index) => index,
			Err(_) => return Err(hash),
		};
		let desc = match self.descs.get(index) {
			Some(desc) => desc,
			None => return Err(hash),
		};
		let name = match name(self.names, desc) {
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
		let &NamesRef { descs, names } = self;
		descs.iter().map(move |desc| (desc.hash, name(names, desc)))
	}

	/// Iterator over the names in this table.
	#[inline]
	pub fn names(&self) -> impl 'a + Clone + Iterator<Item = Option<&'a str>> {
		let &NamesRef { descs, names } = self;
		descs.iter().map(move |desc| name(names, desc))
	}
}

#[inline]
fn name<'a>(names: &'a [u8], desc: &format::NameDesc) -> Option<&'a str> {
	let name = names.get(desc.start as usize..desc.end as usize)?;
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

#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct NameOrHash<'a>(pub Result<&'a str, u32>);

impl<'a> fmt::Display for NameOrHash<'a> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self.0 {
			Ok(name) => write!(f, "{:?}", name),
			Err(hash) => write!(f, "{:#010x}", hash),
		}
	}
}


pub(crate) fn encode_datatable(ds: &mut Dataset, names: &[&str]) {
	// Calculate storage in bytes
	let mut data_size = mem::size_of::<format::NameDesc>() * names.len();
	for &name in names {
		data_size += name.len() + 1;
	}

	// Allocate storage
	let old_len = ds.storage.len();
	let new_len = old_len + (data_size.wrapping_sub(1) / 8).wrapping_add(1);
	ds.storage.resize(new_len, 0);
	let storage = ds.storage[old_len..].as_bytes_mut();

	let (desc, strings) = storage.split_at_mut(mem::size_of::<format::NameDesc>() * names.len());
	let desc = unsafe { slice::from_raw_parts_mut(desc.as_mut_ptr() as *mut format::NameDesc, names.len()) };

	// Write name table to storage
	let mut start = 0;
	for (i, &name) in names.iter().enumerate() {
		let end = start + name.len();
		desc[i].hash = hash(name);
		desc[i].start = start as u16;
		desc[i].end = end as u16;
		strings[start..end].copy_from_slice(name.as_bytes());
		start = end + 1;
	}

	// Sort the name descriptors by hashed name
	desc.sort_unstable_by_key(|desc| desc.hash);

	// Add special name datatable
	ds.tables.push(format::TableDesc {
		key_name: 0,
		type_info: format::TYPE_NAMES,
		compress_info: 0,
		mem_start: old_len as u32,
		mem_end: new_len as u32,
		data_size: data_size as u32,
		data_shape: [names.len() as u32, 0],
		index_name: 0,
		related_name: 0,
		reserved: [0; 3],
	});
	ds.header.len += 1;
}
