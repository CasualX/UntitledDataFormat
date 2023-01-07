use crate::*;

/// Helper for converting data into `DataRef`.
pub trait AsDataRef {
	/// Returns `self` wrapped in a `DataRef`.
	fn as_data_ref(&self) -> DataRef<'_>;
}

macro_rules! impl_as_data_ref_prim {
	($ty:ty, $prim_type:ident) => {
		impl AsDataRef for $ty {
			#[inline]
			fn as_data_ref(&self) -> DataRef<'_> {
				DataRef {
					bytes: dataview::bytes(self),
					type_info: format::TYPE_HINT_NONE | format::TYPE_DIM_SCALAR | format::$prim_type,
					compress_info: format::COMPRESS_NONE,
					shape: [1, 0],
				}
			}
		}
		impl AsDataRef for [$ty] {
			#[inline]
			fn as_data_ref(&self) -> DataRef<'_> {
				DataRef {
					bytes: dataview::bytes(self),
					type_info: format::TYPE_HINT_NONE | format::TYPE_DIM_1D | format::$prim_type,
					compress_info: format::COMPRESS_NONE,
					shape: [self.len() as u32, 0],
				}
			}
		}
		impl<const LEN: usize> AsDataRef for [$ty; LEN] {
			#[inline]
			fn as_data_ref(&self) -> DataRef<'_> {
				DataRef {
					bytes: dataview::bytes(self),
					type_info: format::TYPE_HINT_NONE | format::TYPE_DIM_SCALAR | format::$prim_type,
					compress_info: format::COMPRESS_NONE,
					shape: [self.len() as u32, 0],
				}
			}
		}
		impl<const LEN: usize> AsDataRef for [[$ty; LEN]] {
			#[inline]
			fn as_data_ref(&self) -> DataRef<'_> {
				assert!(LEN < 0x1000000);
				DataRef {
					bytes: dataview::bytes(self),
					type_info: format::TYPE_HINT_NONE | format::TYPE_DIM_1D | format::$prim_type,
					compress_info: format::COMPRESS_NONE,
					shape: [self.len() as u32, LEN as u32],
				}
			}
		}
		impl<const N: usize, const M: usize> AsDataRef for [[$ty; M]; N] {
			#[inline]
			fn as_data_ref(&self) -> DataRef<'_> {
				assert!(M < 0x1000000);
				DataRef {
					bytes: dataview::bytes(self),
					type_info: format::TYPE_HINT_NONE | format::TYPE_DIM_SCALAR | format::$prim_type,
					compress_info: format::COMPRESS_NONE,
					shape: [N as u32, M as u32],
				}
			}
		}
		impl<const N: usize, const M: usize> AsDataRef for [[[$ty; M]; N]] {
			#[inline]
			fn as_data_ref(&self) -> DataRef<'_> {
				assert!(N < 0x1000000);
				assert!(M < 0x100);
				DataRef {
					bytes: dataview::bytes(self),
					type_info: format::TYPE_HINT_NONE | format::TYPE_DIM_1D | format::$prim_type,
					compress_info: format::COMPRESS_NONE,
					shape: [self.len() as u32, N as u32 | (M as u32) << 24],
				}
			}
		}
	};
}

impl_as_data_ref_prim!(u8, TYPE_PRIM_U8);
impl_as_data_ref_prim!(i8, TYPE_PRIM_I8);
impl_as_data_ref_prim!(u16, TYPE_PRIM_U16);
impl_as_data_ref_prim!(i16, TYPE_PRIM_I16);
impl_as_data_ref_prim!(u32, TYPE_PRIM_U32);
impl_as_data_ref_prim!(i32, TYPE_PRIM_I32);
impl_as_data_ref_prim!(u64, TYPE_PRIM_U64);
impl_as_data_ref_prim!(i64, TYPE_PRIM_I64);
impl_as_data_ref_prim!(f32, TYPE_PRIM_F32);
impl_as_data_ref_prim!(f64, TYPE_PRIM_F64);



macro_rules! impl_as_data_ref_typed {
	($ty:ty, $elts:literal, $hint:ident, $prim:ident) => {
		impl AsDataRef for $ty {
			#[inline]
			fn as_data_ref(&self) -> DataRef<'_> {
				DataRef {
					bytes: dataview::bytes(self),
					type_info: format::$hint | format::TYPE_DIM_SCALAR | format::$prim,
					compress_info: format::COMPRESS_NONE,
					shape: [$elts, 0],
				}
			}
		}
		impl AsDataRef for [$ty] {
			#[inline]
			fn as_data_ref(&self) -> DataRef<'_> {
				DataRef {
					bytes: dataview::bytes(self),
					type_info: format::$hint | format::TYPE_DIM_1D | format::$prim,
					compress_info: format::COMPRESS_NONE,
					shape: [self.len() as u32, $elts],
				}
			}
		}
		impl<const LEN: usize> AsDataRef for [$ty; LEN] {
			#[inline]
			fn as_data_ref(&self) -> DataRef<'_> {
				DataRef {
					bytes: dataview::bytes(self),
					type_info: format::$hint | format::TYPE_DIM_1D | format::$prim,
					compress_info: format::COMPRESS_NONE,
					shape: [self.len() as u32, $elts],
				}
			}
		}
	};
}

impl_as_data_ref_typed!(format::FileOffset, 2, TYPE_HINT_DATASET, TYPE_PRIM_U64);
impl_as_data_ref_typed!(format::IndexU32, 0, TYPE_HINT_INDEX, TYPE_PRIM_U32);
impl_as_data_ref_typed!(format::Index2U32, 2, TYPE_HINT_INDEX, TYPE_PRIM_U32);
impl_as_data_ref_typed!(format::Index3U32, 3, TYPE_HINT_INDEX, TYPE_PRIM_U32);
impl_as_data_ref_typed!(format::RangeU32, 2, TYPE_HINT_RANGE, TYPE_PRIM_U32);
impl_as_data_ref_typed!(format::Coord2F32, 2, TYPE_HINT_COORD, TYPE_PRIM_F32);
impl_as_data_ref_typed!(format::Coord3F32, 3, TYPE_HINT_COORD, TYPE_PRIM_F32);
impl_as_data_ref_typed!(format::HatchF32, 4, TYPE_HINT_HATCH, TYPE_PRIM_F32);
impl_as_data_ref_typed!(format::Transform2F32, 6, TYPE_HINT_TRANSFORM, TYPE_PRIM_F32);
impl_as_data_ref_typed!(format::Transform3F32, 12, TYPE_HINT_TRANSFORM, TYPE_PRIM_F32);

impl AsDataRef for str {
	#[inline]
	fn as_data_ref(&self) -> DataRef<'_> {
		DataRef {
			bytes: self.as_bytes(),
			type_info: format::TYPE_PRIM_U8 | format::TYPE_DIM_SCALAR | format::TYPE_HINT_TEXT,
			compress_info: 0,
			shape: [self.len() as u32, 0],
		}
	}
}
