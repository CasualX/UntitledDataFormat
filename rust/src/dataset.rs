use std::mem;
use crate::*;

/// In-memory Dataset.
#[derive(Clone, Default)]
pub struct Dataset {
	pub header: format::DatasetHeader,
	pub descs: Vec<format::TableDesc>,
	pub names: Names,
	pub storage: Vec<u64>,
}

impl Dataset {
	#[inline]
	pub fn new() -> Dataset {
		Dataset {
			header: format::DatasetHeader::default(),
			descs: Vec::new(),
			names: Names::default(),
			storage: Vec::new(),
		}
	}

	#[inline]
	pub fn as_ref(&self) -> DatasetRef<'_> {
		DatasetRef {
			header: &self.header,
			storage: &self.storage,
			names: self.names.as_ref(),
			tables: &self.descs,
		}
	}

	/// The number of tables in this dataset.
	#[inline]
	pub fn len(&self) -> usize {
		self.descs.len()
	}

	/// Adds a new table.
	///
	/// This adds a table record and copies the data to the internal storage.
	///
	/// If another table already exists with the key name returns `false` and does not insert the table.
	pub fn add_table(&mut self, table_ref: TableRef) -> bool {
		let storage = table_ref.data.bytes;
		let (mem_start, mem_end) = self.write_data(storage);

		let key_name = table_ref.key_name;
		let index_name = table_ref.index_name;
		let rel_name = table_ref.related_name;
		let type_info = table_ref.data.type_info;
		let compress_info = table_ref.data.compress_info;
		let data_size = table_ref.data.bytes.len() as u32;
		let data_shape = table_ref.data.shape.encode();

		self.descs.push(format::TableDesc {
			key_name,
			type_info,
			compress_info,
			mem_start, mem_end,
			data_size, data_shape,
			index_name,
			related_name: rel_name,
			type_name: 0,
			checksum: 0,
			reserved: [0; 1],
		});
		return true;
	}

	fn write_data(&mut self, storage: &[u8]) -> (u32, u32) {
		if storage.is_empty() {
			return (0, 0);
		}

		let offset = self.storage.len() * 8;

		// Allocate zeroed data storage
		let old_len = self.storage.len();
		let new_len = old_len + ((storage.len() - 1) / 8) + 1;
		self.storage.resize(new_len, 0);

		// Copy data into the storage
		let dest = &mut dataview::bytes_mut(self.storage.as_mut_slice())[offset..offset + storage.len()];
		dest.copy_from_slice(storage);

		return (old_len as u32, new_len as u32);
	}

	pub fn finalize(&mut self) -> Final<DatasetRef> {
		self.header.check = format::DatasetHeader::CHECK;
		self.names.finalize();

		let mut size = 0;
		size += mem::size_of_val(&self.header);
		size += mem::size_of_val(&self.descs[..]);
		size += self.names.as_ref().file_size();
		debug_assert_eq!(size % 8, 0);
		self.header.size = size as u16;

		self.header.descs_len = self.descs.len() as u16;
		self.header.lookup_len = self.names.entries.len() as u16;
		self.header.string_len = self.names.strings.len() as u16;

		Final { inner: self.as_ref() }
	}
}
