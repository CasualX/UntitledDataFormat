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
	header: format::Header,
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
		let header = format::Header {
			magic: format::Header::MAGIC,
			id,
			next: mem::size_of::<format::Header>() as u64,
			root: format::FileOffset::default(),
			temp: [0; 4],
		};
		file.write_all(header.as_bytes())?;
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
		let mut header = format::Header::default();
		file.read_exact(header.as_bytes_mut())?;
		if header.magic != format::Header::MAGIC {
			return Err(io::Error::from(io::ErrorKind::InvalidData));
		}
		Ok(FileIO { file, header })
	}

	/// Opens an UDF file for editing.
	///
	/// Fails if the file does not yet exist.
	#[inline]
	pub fn edit(path: impl AsRef<Path>) -> io::Result<FileIO> {
		Self::edit_(path.as_ref())
	}
	fn edit_(path: &Path) -> io::Result<FileIO> {
		let mut file = fs::OpenOptions::new().read(true).write(true).open(path)?;
		let mut header = format::Header::default();
		file.read_exact(header.as_bytes_mut())?;
		if header.magic != format::Header::MAGIC {
			return Err(io::Error::from(io::ErrorKind::InvalidData));
		}
		Ok(FileIO { file, header })
	}

	/// Returns the file id.
	pub fn id(&self) -> [u8; 4] {
		self.header.id
	}

	/// Sets the file id.
	pub fn set_id(&mut self, id: [u8; 4]) {
		self.header.id = id;
	}

	/// Returns the root dataset.
	pub fn root(&self) -> format::FileOffset {
		self.header.root
	}

	/// Sets the root dataset.
	pub fn set_root(&mut self, root: format::FileOffset) {
		self.header.root = root;
	}

	/// Writes the updated header to the file.
	pub fn write_header(&mut self) -> io::Result<()> {
		// Before writing the header, sync other changes
		self.file.flush()?;
		self.file.seek(io::SeekFrom::Start(0))?;
		self.file.write_all(self.header.as_bytes())?;
		Ok(())
	}

	/// Allocates a file offset.
	pub fn allocate(&mut self, size: usize) -> format::FileOffset {
		let offset = self.header.next;
		let size = (size.wrapping_sub(1) & !0xf).wrapping_add(0x10) as u64;
		self.header.next += size;
		format::FileOffset { offset, size }
	}

	/// Adds a dataset to the UDF file.
	///
	/// A new section is allocated for the dataset.
	pub fn add_dataset(&mut self, ds: DatasetRef<'_>) -> io::Result<format::FileOffset> {
		let fo = self.allocate(ds.file_size());
		self.write_dataset(fo, ds)?;
		Ok(fo)
	}

	/// Writes a dataset to the UDF file.
	///
	/// This API lets you specify an arbitrary file offset.
	/// Needless to say it is trivial to corrupt the UDF file with this.
	/// Only use this with values retrieved from [`allocate`](Self::allocate).
	pub fn write_dataset(&mut self, fo: format::FileOffset, ds: DatasetRef<'_>) -> io::Result<()> {
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
		self.file.write_all(ds.header.as_bytes())?;
		self.file.write_all(ds.tables.as_bytes())?;
		self.file.write_all(ds.storage.as_bytes())?;

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
		if fo.offset == 0 || fo.size == 0 || fo.offset & 0xf != 0 || fo.size & 0xf != 0 {
			return Err(io::Error::from(io::ErrorKind::InvalidInput));
		}

		let mut header = format::DatasetHeader::default();
		self.file.seek(io::SeekFrom::Start(fo.offset))?;
		self.file.read_exact(header.as_bytes_mut())?;

		let header_size = mem::size_of::<format::DatasetHeader>() + mem::size_of::<format::Table>() * header.len as usize;
		if header.check != format::DatasetHeader::CHECK || fo.size < header_size as u64 {
			return Err(io::Error::from(io::ErrorKind::InvalidData));
		}

		let mut tables = vec![format::Table::default(); header.len as usize];
		let mut storage = vec![0u64; (fo.size as usize - header_size) / 8];
		self.file.read_exact(tables.as_bytes_mut())?;
		self.file.read_exact(storage.as_bytes_mut())?;

		Ok(Dataset { header, tables, storage })
	}

	pub fn flush(&mut self) -> io::Result<()> {
		self.file.flush()
	}
}

static ZEROS: [u8; 512] = [0; 512];
