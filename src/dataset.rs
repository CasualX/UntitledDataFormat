use std::mem;
use crate::*;

/// Dataset by reference.
#[derive(Copy, Clone, Debug)]
pub struct DatasetRef<'a> {
	pub header: &'a format::DatasetHeader,
	pub tables: &'a [format::TableDesc],
	pub storage: &'a [u64],
}

impl<'a> DatasetRef<'a> {
	/// Parses the dataset from the storage itself.
	pub fn parse(storage: &'a [u64]) -> Result<DatasetRef<'a>, ParseError> {
		let data_view = storage.as_data_view();
		let header = match data_view.try_read::<format::DatasetHeader>(0) {
			Some(header) => header,
			None => return Err(ParseError::OutOfBounds),
		};

		let tables_len = usize::max(header.len as usize, header.max_len as usize);
		let tables = match data_view.try_slice::<format::TableDesc>(mem::size_of_val(header), tables_len) {
			Some(tables) => tables,
			None => return Err(ParseError::OutOfBounds),
		};

		let start = mem::size_of_val(header) / 8 + mem::size_of_val(tables) / 8;
		let storage = match storage.get(start..) {
			Some(storage) => storage,
			None => return Err(ParseError::OutOfBounds),
		};

		return Ok(DatasetRef { header, tables, storage });
	}

	/// The number of tables in this dataset.
	#[inline]
	pub fn len(&self) -> usize {
		self.tables.len()
	}

	/// Returns the file size in bytes that this dataset requires.
	#[inline]
	pub fn file_size(&self) -> usize {
		mem::size_of_val(self.header) + mem::size_of_val(self.storage) + mem::size_of_val(self.tables)
	}

	/// Gets the special names table.
	///
	/// This table allows lookup of hashed keys to their string representation.
	///
	/// Returns an empty `NamesRef` if absent or there was an error reading the names table.
	pub fn get_names(&self) -> NamesRef<'a> {
		// Names table must be the first entry
		let table = match self.tables.get(0) {
			Some(table) => table,
			None => return NamesRef::new(),
		};
		// Check if it really is the names table
		if table.key_name != 0 || table.type_info != format::TYPE_NAMES {
			return NamesRef::new();
		}

		let data = match self.storage.get(table.mem_start as usize..table.mem_end as usize) {
			Some(storage) => storage.as_bytes(),
			None => return NamesRef::new(),
		};

		let len = table.data_shape[0] as usize;
		match NamesRef::from_data(data, len) {
			Some(names) => names,
			None => NamesRef::new()
		}
	}

	pub fn get_name(&self, name: u32) -> NameOrHash<'a> {
		unimplemented!()
	}

	/// Finds a table descriptor by its key name.
	#[inline]
	pub fn find_table(&self, key_name: u32) -> Option<&format::TableDesc> {
		self.tables.iter().find(move |&table| table.key_name == key_name)
		// match self.tables.binary_search_by_key(&key_name, |table| table.key_name) {
		// 	Ok(index) => Some(&self.tables[index]),
		// 	Err(_) => None,
		// }
	}

	#[inline]
	pub fn get_data_ref(&self, table: &format::TableDesc) -> Option<DataRef<'a>> {
		let storage = self.storage.get(table.mem_start as usize..table.mem_end as usize)?;
		let bytes = storage.as_bytes().get(..table.data_size as usize)?;
		let type_info = table.type_info;
		let compress_info = table.compress_info;
		let shape = table.data_shape;
		Some(DataRef { bytes, type_info, compress_info, shape })
	}
}


/// In-memory Dataset.
#[derive(Clone)]
pub struct Dataset {
	pub header: format::DatasetHeader,
	pub tables: Vec<format::TableDesc>,
	pub storage: Vec<u64>,
}

impl Dataset {
	#[inline]
	pub fn create(names: &[&str]) -> Dataset {
		let mut ds = Dataset {
			header: format::DatasetHeader { check: format::DatasetHeader::CHECK, ..Default::default() },
			tables: Vec::new(),
			storage: Vec::new(),
		};
		ds.set_names(names);
		return ds;
	}

	#[inline]
	pub fn as_ref(&self) -> DatasetRef<'_> {
		DatasetRef {
			header: &self.header,
			storage: &self.storage,
			tables: &self.tables,
		}
	}

	/// The number of tables in this dataset.
	#[inline]
	pub fn len(&self) -> usize {
		self.tables.len()
	}

	/// Adds a special names table.
	///
	/// This table allows lookup of hashed keys to their string representation.
	#[inline]
	fn set_names(&mut self, names: &[&str]) {
		crate::names::encode_datatable(self, names)
	}

	/// Gets the special names table.
	///
	/// This table allows lookup of hashed keys to their string representation.
	///
	/// Returns `None` if absent or there was an error reading the names table.
	#[inline]
	pub fn get_names(&self) -> NamesRef<'_> {
		self.as_ref().get_names()
	}

	/// Adds a new table.
	///
	/// This adds a table record and copies the data to the internal storage.
	///
	/// If another table already exists with the key name returns `false` and does not insert the table.
	pub fn add_table(&mut self, table_ref: TableRef) -> bool {
		// Reserved for names table
		if table_ref.key_name == 0 {
			return false;
		}

		let storage = table_ref.data.bytes.as_bytes();
		let (mem_start, mem_end) = self.write_data(storage);

		let place = match self.tables.binary_search_by_key(&table_ref.key_name, |desc| desc.key_name) {
			Ok(_) => return false,
			Err(place) => place,
		};

		let key_name = table_ref.key_name;
		let index_name = table_ref.index_name;
		let rel_name = table_ref.related_name;
		let type_info = table_ref.data.type_info;
		let data_size = table_ref.data.bytes.len() as u32;
		let data_shape = table_ref.data.shape;

		self.tables.insert(place, format::TableDesc {
			key_name,
			type_info,
			compress_info: 0,
			mem_start, mem_end,
			data_size, data_shape,
			index_name,
			related_name: rel_name,
			reserved: [0; 3],
		});
		self.header.len += 1;
		return true;
	}

	fn write_data(&mut self, storage: &[u8]) -> (u32, u32) {
		let offset = self.storage.len() * 8;

		// Allocate zeroed data storage
		let old_len = self.storage.len();
		let new_len = old_len + (storage.len().wrapping_sub(1) / 8).wrapping_add(1);
		self.storage.resize(new_len, 0);

		// Copy data into the storage
		let dest = &mut self.storage.as_bytes_mut()[offset..offset + storage.len()];
		dest.copy_from_slice(storage);

		return (old_len as u32, new_len as u32);
	}
}
