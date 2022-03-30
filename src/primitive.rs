
#[derive(Copy, Clone)]
pub enum PrimRef<'a> {
	U8(&'a [u8]),
	I8(&'a [i8]),
	U16(&'a [u16]),
	I16(&'a [i16]),
	U32(&'a [u32]),
	I32(&'a [i32]),
	U64(&'a [u64]),
	I64(&'a [i64]),
	FP16(&'a [u16]),
	F32(&'a [f32]),
	F64(&'a [f64]),
}

impl<'a> Default for PrimRef<'a> {
	#[inline]
	fn default() -> PrimRef<'a> {
		PrimRef::U8(&[])
	}
}

impl<'a> PrimRef<'a> {
	#[inline]
	pub fn type_name(self) -> &'static str {
		match self {
			PrimRef::U8(_) => "U8",
			PrimRef::I8(_) => "I8",
			PrimRef::U16(_) => "U16",
			PrimRef::I16(_) => "I16",
			PrimRef::U32(_) => "U32",
			PrimRef::I32(_) => "I32",
			PrimRef::U64(_) => "U64",
			PrimRef::I64(_) => "I64",
			PrimRef::FP16(_) => "FP16",
			PrimRef::F32(_) => "F32",
			PrimRef::F64(_) => "F64",
		}
	}

	#[inline]
	pub fn primitive_type(self) -> u16 {
		match self {
			PrimRef::U8(_) => format::TYPE_PRIM_U8,
			PrimRef::I8(_) => format::TYPE_PRIM_I8,
			PrimRef::U16(_) => format::TYPE_PRIM_U16,
			PrimRef::I16(_) => format::TYPE_PRIM_I16,
			PrimRef::U32(_) => format::TYPE_PRIM_U32,
			PrimRef::I32(_) => format::TYPE_PRIM_I32,
			PrimRef::U64(_) => format::TYPE_PRIM_U64,
			PrimRef::I64(_) => format::TYPE_PRIM_I64,
			PrimRef::FP16(_) => format::TYPE_PRIM_BFLOAT16,
			PrimRef::F32(_) => format::TYPE_PRIM_F32,
			PrimRef::F64(_) => format::TYPE_PRIM_F64,
		}
	}

	#[inline]
	pub fn as_bytes(self) -> &'a [u8] {
		match self {
			PrimRef::U8(data) => data,
			PrimRef::I8(data) => data.as_bytes(),
			PrimRef::U16(data) => data.as_bytes(),
			PrimRef::I16(data) => data.as_bytes(),
			PrimRef::U32(data) => data.as_bytes(),
			PrimRef::I32(data) => data.as_bytes(),
			PrimRef::U64(data) => data.as_bytes(),
			PrimRef::I64(data) => data.as_bytes(),
			PrimRef::FP16(data) => data.as_bytes(),
			PrimRef::F32(data) => data.as_bytes(),
			PrimRef::F64(data) => data.as_bytes(),
		}
	}

	#[inline]
	pub fn len(self) -> usize {
		match self {
			PrimRef::U8(data) => data.len(),
			PrimRef::I8(data) => data.len(),
			PrimRef::U16(data) => data.len(),
			PrimRef::I16(data) => data.len(),
			PrimRef::U32(data) => data.len(),
			PrimRef::I32(data) => data.len(),
			PrimRef::U64(data) => data.len(),
			PrimRef::I64(data) => data.len(),
			PrimRef::FP16(data) => data.len(),
			PrimRef::F32(data) => data.len(),
			PrimRef::F64(data) => data.len(),
		}
	}
}
