use std::{fmt, mem, slice};
use crate::*;

/// Data reference.
#[derive(Copy, Clone, Default)]
pub struct DataRef<'a> {
	/// Data bytes.
	pub bytes: &'a [u8],
	/// Type primitive, hint and dimensions.
	pub type_info: u16,
	/// Compression applied to the data bytes.
	pub compress_info: u16,
	/// Length values of up to 3 dimensions.
	pub shape: Shape,
}

impl<'a> DataRef<'a> {
	/// Returns the total number of elements.
	#[inline]
	pub fn len(&self) -> usize {
		self.shape.len()
	}

	/// Returns whether the data is compressed.
	#[inline]
	pub fn is_compressed(&self) -> bool {
		self.compress_info != format::COMPRESS_NONE
	}

	/// Reinterpret the data.
	///
	/// The data bytes must have the correct alignment and size or this cast fails.
	///
	/// The data bytes must not be compressed or this cast fails.
	pub fn as_slice<T: dataview::Pod>(&self) -> Option<&'a [T]> {
		if self.compress_info != format::COMPRESS_NONE {
			return None;
		}
		let bytes = self.bytes;
		if bytes.as_ptr() as usize % mem::align_of::<T>() != 0 {
			return None;
		}
		if bytes.len() % mem::size_of::<T>() != 0 {
			return None;
		}
		let len = bytes.len() / mem::size_of::<T>();
		let data = bytes.as_ptr() as *const T;
		unsafe { Some(slice::from_raw_parts(data, len)) }
	}

	/// Returns the data as a printable array.
	///
	/// This operation fails if the data is invalid (as_slice fails).
	#[inline(never)]
	pub fn print(&self) -> Result<PrintArray, fmt::Error> {
		// Cannot print compressed data
		if self.compress_info != 0 {
			return Err(fmt::Error);
		}

		let len = self.shape.len();
		let avg_size;

		// Concrete iterator instances
		let mut iter_u8;
		let mut iter_i8;
		let mut iter_u16;
		let mut iter_i16;
		let mut iter_u32;
		let mut iter_i32;
		let mut iter_u64;
		let mut iter_i64;
		let mut iter_f32;
		let mut iter_f64;

		// Generic interface
		let items: &mut dyn Iterator<Item = &dyn fmt::Display>;
		items = match self.type_info & format::TYPE_PRIM_MASK {
			format::TYPE_PRIM_U8 => {
				avg_size = 1;
				let values = self.as_slice::<u8>().ok_or(fmt::Error)?;
				iter_u8 = values.iter().map(|v| v as _);
				&mut iter_u8
			},
			format::TYPE_PRIM_I8 => {
				avg_size = 1;
				let values = self.as_slice::<i8>().ok_or(fmt::Error)?;
				iter_i8 = values.iter().map(|v| v as _);
				&mut iter_i8
			},
			format::TYPE_PRIM_U16 => {
				avg_size = 1;
				let values = self.as_slice::<u16>().ok_or(fmt::Error)?;
				iter_u16 = values.iter().map(|v| v as _);
				&mut iter_u16
			},
			format::TYPE_PRIM_I16 => {
				avg_size = 1;
				let values = self.as_slice::<i16>().ok_or(fmt::Error)?;
				iter_i16 = values.iter().map(|v| v as _);
				&mut iter_i16
			},
			format::TYPE_PRIM_U32 => {
				avg_size = 2;
				let values = self.as_slice::<u32>().ok_or(fmt::Error)?;
				iter_u32 = values.iter().map(|v| v as _);
				&mut iter_u32
			},
			format::TYPE_PRIM_I32 => {
				avg_size = 2;
				let values = self.as_slice::<i32>().ok_or(fmt::Error)?;
				iter_i32 = values.iter().map(|v| v as _);
				&mut iter_i32
			},
			format::TYPE_PRIM_U64 => {
				avg_size = 2;
				let values = self.as_slice::<u64>().ok_or(fmt::Error)?;
				iter_u64 = values.iter().map(|v| v as _);
				&mut iter_u64
			},
			format::TYPE_PRIM_I64 => {
				avg_size = 2;
				let values = self.as_slice::<i64>().ok_or(fmt::Error)?;
				iter_i64 = values.iter().map(|v| v as _);
				&mut iter_i64
			},
			format::TYPE_PRIM_F32 => {
				avg_size = 5;
				let values = self.as_slice::<f32>().ok_or(fmt::Error)?;
				iter_f32 = values.iter().map(|v| PrintF32::wrap(v) as _);
				&mut iter_f32
			},
			format::TYPE_PRIM_F64 => {
				avg_size = 8;
				let values = self.as_slice::<f64>().ok_or(fmt::Error)?;
				iter_f64 = values.iter().map(|v| PrintF64::wrap(v) as _);
				&mut iter_f64
			},
			_ => return Err(fmt::Error),
		};

		let mut pa = PrintArray::new(self.shape);
		pa.reserve(len, avg_size);
		for item in items {
			pa.push_fmt(format_args!("{}", item))?;
		}
		Ok(pa)
	}

	/// Decompress the data.
	pub fn decompress(&self, storage: &'a mut Vec<u64>) -> DataRef<'a> {
		match self.compress_info {
			format::COMPRESS_SIMPLE_U32 => {
				let len = self.shape.len();
				storage.resize_with(len / 2 + 1, Default::default);
				let view_mut = dataview::DataView::from_mut(storage.as_mut_slice());
				let tail_len = view_mut.tail_len::<u32>(0);
				let storage = &mut view_mut.slice_mut(0, tail_len)[..len];
				if compress::SimpleU32::decompress(storage, self.bytes) {
					return DataRef {
						bytes: dataview::bytes(storage),
						compress_info: format::COMPRESS_NONE,
						shape: self.shape,
						type_info: self.type_info,
					};
				}
			},
			format::COMPRESS_SIMPLE_F32 => {
				let len = self.shape.len();
				storage.resize_with(len / 2 + 1, Default::default);
				let view_mut = dataview::DataView::from_mut(storage.as_mut_slice());
				let tail_len = view_mut.tail_len::<f32>(0);
				let storage = &mut view_mut.slice_mut(0, tail_len)[..len];
				if compress::SimpleF32::decompress(storage, self.bytes) {
					return DataRef {
						bytes: dataview::bytes(storage),
						compress_info: format::COMPRESS_NONE,
						shape: self.shape,
						type_info: self.type_info,
					};
				}
			}
			_ => (),
		}
		// Not compressed or there was an error decompressing
		return *self;
	}
}
