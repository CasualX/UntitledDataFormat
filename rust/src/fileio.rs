/*!
Standard file writer.
*/

use std::{fs, mem};
use std::path::Path;
use std::io::{self, Read, Seek, Write};
use crate::*;

/// Manipulate UDF files through File IO.
pub struct FileIO {
	file: fs::File,
	header: format::UdfHeader,
}

impl FileIO {
	/// Creates a new UDF file.
	///
	/// Creates a new file if it does not exists, and will truncate if it does.
	#[inline]
	pub fn create(path: impl AsRef<Path>, id: [u8; 4]) -> io::Result<FileIO> {
		Self::create_(path.as_ref(), id)
	}
	fn create_(path: &Path, id: [u8; 4]) -> io::Result<FileIO> {
		let mut file = fs::File::create(path)?;
		let header = format::UdfHeader {
			magic: format::UdfHeader::MAGIC,
			id,
			next: 0,
			root: format::FileOffset::default(),
			reserved: [0; 4],
		};
		file.write_all(dataview::bytes(&header))?;
		Ok(FileIO { file, header })
	}

	/// Opens an UDF file in read-only mode.
	///
	/// Write methods will return an error.
	#[inline]
	pub fn open(path: impl AsRef<Path>) -> io::Result<FileIO> {
		Self::open_(path.as_ref())
	}
	fn open_(path: &Path) -> io::Result<FileIO> {
		let mut file = fs::File::open(path)?;
		let mut header = format::UdfHeader::default();
		file.read_exact(dataview::bytes_mut(&mut header))?;
		if header.magic != format::UdfHeader::MAGIC {
			return Err(io::Error::from(io::ErrorKind::InvalidData));
		}
		Ok(FileIO { file, header })
	}

	/// Opens an UDF file for editing.
	///
	/// Creates a new file if it does not exists.
	#[inline]
	pub fn edit(path: impl AsRef<Path>) -> io::Result<FileIO> {
		Self::edit_(path.as_ref())
	}
	fn edit_(path: &Path) -> io::Result<FileIO> {
		let mut file = fs::OpenOptions::new().read(true).write(true).open(path)?;
		let mut header = format::UdfHeader::default();
		file.read_exact(dataview::bytes_mut(&mut header))?;
		if header.magic != format::UdfHeader::MAGIC {
			return Err(io::Error::from(io::ErrorKind::InvalidData));
		}
		Ok(FileIO { file, header })
	}

	/// Returns the file id.
	pub fn id(&self) -> [u8; 4] {
		self.header.id
	}

	/// Sets the file id.
	///
	/// Invoke [`write_header`](Self::write_header) to persist the change.
	pub fn set_id(&mut self, id: [u8; 4]) {
		self.header.id = id;
	}

	/// Returns the root dataset.
	pub fn root(&self) -> format::FileOffset {
		self.header.root
	}

	/// Sets the root dataset.
	///
	/// Invoke [`write_header`](Self::write_header) to persist the change.
	pub fn set_root(&mut self, root: format::FileOffset) {
		self.header.root = root;
	}

	/// Writes the updated header to the file.
	pub fn write_header(&mut self) -> io::Result<()> {
		self.file.seek(io::SeekFrom::Start(0))?;
		self.file.write_all(dataview::bytes(&self.header))?;
		Ok(())
	}

	/// Allocates a file offset.
	pub fn allocate(&mut self, size: usize) -> format::FileOffset {
		let offset = match self.file.metadata() {
			Ok(md) => md.len(),
			Err(_) => return Default::default(),
		};
		let offset = (offset.wrapping_sub(1) & !0xf).wrapping_add(0x10);
		let size = (size.wrapping_sub(1) & !0xf).wrapping_add(0x10) as u64;
		format::FileOffset { offset, size }
	}

	/// Adds a dataset to the UDF file.
	///
	/// A new section is allocated for the dataset.
	pub fn add_dataset(&mut self, ds: &Final<DatasetRef<'_>>) -> io::Result<format::FileOffset> {
		let fo = self.allocate(ds.inner.file_size());
		self.write_dataset(fo, ds)?;
		Ok(fo)
	}

	/// Writes a dataset to the UDF file.
	///
	/// This API lets you specify an arbitrary file offset.
	/// Needless to say it is trivial to corrupt the UDF file with this.
	/// Only use this with values retrieved from [`allocate`](Self::allocate).
	pub fn write_dataset(&mut self, fo: format::FileOffset, ds: &Final<DatasetRef<'_>>) -> io::Result<()> {
		let ds = &ds.inner;

		// File offsets must be 16-byte aligned
		if fo.is_null() || !fo.is_aligned() {
			return Err(io::Error::from(io::ErrorKind::InvalidInput));
		}

		// Must have enough space to hold the dataset
		let ds_file_size = ds.file_size() as u64;
		if fo.size < ds_file_size {
			return Err(io::Error::from(io::ErrorKind::InvalidInput));
		}

		// Write the dataset to the file
		self.file.seek(io::SeekFrom::Start(fo.offset))?;
		ds.write(&mut self.file)?;

		// Write zeros to the remaining data
		let mut remaining_zeros = fo.size - ds_file_size;
		while remaining_zeros > 0 {
			let zeros_len = u64::min(mem::size_of_val(&ZEROS) as u64, remaining_zeros) as usize;
			self.file.write_all(&ZEROS[..zeros_len])?;
			remaining_zeros -= zeros_len as u64;
		}
		Ok(())
	}

	/// Reads a dataset from the UDF file.
	///
	/// This API lets you specify an arbitrary file offset.
	/// It should only be used with file offsets retrieved from [`allocate`](Self::allocate).
	/// However there's some safety checks against reading arbitrary data as datasets.
	pub fn read_dataset(&mut self, fo: format::FileOffset) -> io::Result<Dataset> {
		// File offsets must be 16-byte aligned
		if fo.is_null() || !fo.is_aligned() {
			return Err(io::Error::from(io::ErrorKind::InvalidInput));
		}

		// Copy the whole dataset in memory
		let mut storage = vec![0u64; (fo.size / 8) as usize];
		self.file.seek(io::SeekFrom::Start(fo.offset))?;
		self.file.read_exact(dataview::bytes_mut(storage.as_mut_slice()))?;

		// println!("{:?}", storage);

		let Ok(ds) = DatasetRef::parse(&storage) else {
			return Err(io::Error::from(io::ErrorKind::InvalidData));
		};

		Ok(ds.to_owned())
	}

	/// Flushes the underlying file object.
	pub fn flush(&mut self) -> io::Result<()> {
		self.file.flush()
	}
}

static ZEROS: [u8; 512] = [0; 512];
