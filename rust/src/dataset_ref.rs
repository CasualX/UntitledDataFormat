use std::{io, mem};
use crate::*;

/// Dataset by reference.
#[derive(Copy, Clone, Debug)]
pub struct DatasetRef<'a> {
	pub header: &'a format::DatasetHeader,
	pub tables: &'a [format::TableDesc],
	pub names: NamesRef<'a>,
	pub storage: &'a [u64],
}

impl<'a> DatasetRef<'a> {
	/// Parses the dataset from the storage itself.
	pub fn parse(storage: &'a [u64]) -> Result<DatasetRef<'a>, ParseError> {
		let view = dataview::DataView::from(storage);
		let Some(header) = view.try_get::<format::DatasetHeader>(0) else {
			return Err(ParseError::OutOfBounds);
		};

		if header.size % 8 != 0 {
			return Err(ParseError::InvalidFormat);
		}

		let Some(head) = view.try_slice::<u8>(0, header.size as usize) else {
			return Err(ParseError::OutOfBounds);
		};

		let view = dataview::DataView::from(head);

		let mut offset = mem::size_of_val(header);
		let Some(descs) = view.try_slice::<format::TableDesc>(offset, header.descs_len as usize) else {
			return Err(ParseError::OutOfBounds);
		};

		offset += mem::size_of_val(descs);
		let Some(entries) = view.try_slice::<format::LookupEntry>(offset, header.lookup_len as usize) else {
			return Err(ParseError::OutOfBounds);
		};

		offset += mem::size_of_val(entries);
		let Some(strings) = view.try_slice::<u8>(offset, header.string_len as usize) else {
			return Err(ParseError::OutOfBounds);
		};

		let names = NamesRef { entries, strings };

		let storage = match storage.get(header.size as usize / 8..) {
			Some(storage) => storage,
			None => return Err(ParseError::OutOfBounds),
		};

		return Ok(DatasetRef { header, tables: descs, names, storage });
	}

	pub fn to_owned(&self) -> Dataset {
		Dataset {
			header: *self.header,
			descs: self.tables.to_owned(),
			names: self.names.to_owned(),
			storage: self.storage.to_owned(),
		}
	}

	/// The number of tables in this dataset.
	#[inline]
	pub fn len(&self) -> usize {
		self.tables.len()
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
		let bytes = dataview::bytes(storage).get(..table.data_size as usize)?;
		let type_info = table.type_info;
		let compress_info = table.compress_info;
		let shape = table.data_shape;
		Some(DataRef { bytes, type_info, compress_info, shape })
	}

	/// Returns the file size in bytes that this dataset requires.
	#[inline]
	pub fn file_size(&self) -> usize {
		mem::size_of_val(self.header) + mem::size_of_val(self.tables) + self.names.file_size() + mem::size_of_val(self.storage)
	}

	pub fn write(&self, w: &mut dyn io::Write) -> io::Result<()> {
		w.write_all(dataview::bytes(self.header))?;
		w.write_all(dataview::bytes(&self.tables[..]))?;
		self.names.write(w)?;
		w.write_all(dataview::bytes(&self.storage[..]))?;
		Ok(())
	}
}
