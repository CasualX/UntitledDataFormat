use std::{fmt, str};
use crate::*;

#[derive(Clone, Default)]
pub struct Names {
	pub entries: Vec<format::LookupEntry>,
	pub strings: Vec<u8>,
}

impl Names {
	#[inline]
	pub fn as_ref(&self) -> NamesRef<'_> {
		NamesRef {
			entries: &self.entries,
			strings: &self.strings,
		}
	}

	#[inline]
	pub fn len(&self) -> usize {
		self.entries.len()
	}

	pub fn add(&mut self, name: &str, hash: u32) {
		let offset = self.strings.len() as u16;
		let len = name.len() as u16;
		self.entries.push(format::LookupEntry { hash, offset, len });
		self.strings.extend_from_slice(name.as_bytes());
	}

	pub fn finalize(&mut self) {
		// Sort the names to enable binary search
		self.entries.sort_unstable_by_key(|name| name.hash);

		// Pad the internal strings buffer to a length multiple of 8
		// This asserts that the whole names block is a multiple of 8
		let len = self.strings.len();
		let new_len = (len.wrapping_sub(1) & !0x7).wrapping_add(8);
		self.strings.resize(new_len, 0u8);
	}
}

impl fmt::Debug for Names {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.as_ref().fmt(f)
	}
}
