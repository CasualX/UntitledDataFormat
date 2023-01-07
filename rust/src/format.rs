/*!
Disk format structs.
*/

use std::mem;

/// The UDF file header.
#[derive(Copy, Clone, Default, dataview::Pod)]
#[repr(C)]
pub struct UdfHeader {
	/// Magic file format identifier.
	///
	/// Must be set to [`MAGIC`](Self::MAGIC).
	pub magic: [u8; 4],
	/// Identifies the file format conventions.
	///
	/// This field can be used to quickly identify if this UDF file is deliberately created for your application.
	pub id: [u8; 4],
	/// Reserved for future use.
	pub next: u64,
	/// The root dataset.
	pub root: FileOffset,
	/// Reserved for future use, must be zero.
	pub reserved: [u64; 4],
}

const _: [(); 0x40] = [(); mem::size_of::<UdfHeader>()];

impl UdfHeader {
	pub const MAGIC: [u8; 4] = *b"UDF0";
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Default, Hash, dataview::Pod)]
#[repr(C)]
pub struct FileOffset {
	/// Absolute offset from the start of the file.
	///
	/// Must be 16-byte aligned.
	pub offset: u64,
	/// Size of the dataset, must contain all the data of the dataset.
	///
	/// Must be 16-byte aligned.
	pub size: u64,
}

impl FileOffset {
	pub const NULL: FileOffset = FileOffset { offset: 0, size: 0 };

	#[inline]
	pub const fn is_null(&self) -> bool {
		self.offset == 0 && self.size == 0
	}
	#[inline]
	pub const fn is_aligned(&self) -> bool {
		self.offset & 0xf == 0 && self.size & 0xf == 0
	}
}

#[derive(Copy, Clone, Debug, Default, dataview::Pod)]
#[repr(C)]
pub struct DatasetHeader {
	/// Check value allows checking if this is really a dataset.
	///
	/// Must be set to [`CHECK`](Self::CHECK).
	pub check: u32,
	/// Checksum of the header (starting from the next field).
	///
	/// If zero, the checksum is absent.
	pub checksum: u32,
	/// Identifies the dataset convention.
	pub id: [u8; 4],
	/// Size of the dataset header in bytes (including tables and strings).
	///
	/// Must be 8-byte aligned.
	pub size: u16,
	/// Number of datatable descriptors following the header.
	pub descs_len: u16,
	/// Number of string lookup entries following the datatable descriptors.
	pub lookup_len: u16,
	/// Byte length of the string following the lookup entries.
	///
	/// Must be 8-byte aligned.
	pub string_len: u16,
	/// Reserved for future use.
	pub reserved: [u16; 2],
}

const _: [(); 0x18] = [(); mem::size_of::<DatasetHeader>()];

impl DatasetHeader {
	pub const CHECK: u32 = 0x7fcea59b;
}

#[derive(Copy, Clone, Debug, Default, dataview::Pod)]
#[repr(C)]
pub struct TableDesc {
	/// Key name of the table.
	pub key_name: u32,
	/// Combined type primitive, hint and flags.
	pub type_info: u16,
	/// Compression applied to the data storage.
	pub compress_info: u16,
	/// Block start index in the storage where the data is stored.
	pub mem_start: u32,
	/// Block end index of the storage reserved for the data.
	pub mem_end: u32,
	/// Size of the data in bytes.
	pub data_size: u32,
	/// Multidimensional shape of the data.
	pub data_shape: [u32; 2],
	/// When an index type hint is used this specifies which datatable the indices go into. Otherwise 0.
	///
	/// If the datatable by this name does not exist in the dataset, walk through parent datasets until found.
	pub index_name: u32,
	/// Struct of Arrays (SoA) indicates related datatable.
	pub related_name: u32,
	/// Extended type information as a string.
	pub type_name: u32,
	/// Checksum of the memory block.
	///
	/// If zero, the checksum is absent.
	pub checksum: u32,
	/// Reserved for future use.
	pub reserved: [u32; 1],
}

const _: [(); 0x30] = [(); mem::size_of::<TableDesc>()];

/// Primitive types.
pub const TYPE_PRIM_MASK: u16 = 0x008f;
pub const TYPE_PRIM_CUSTOM: u16 = 0;
pub const TYPE_PRIM_U8: u16 = 2;
pub const TYPE_PRIM_I8: u16 = 3;
pub const TYPE_PRIM_U16: u16 = 4;
pub const TYPE_PRIM_I16: u16 = 5;
pub const TYPE_PRIM_U32: u16 = 6;
pub const TYPE_PRIM_I32: u16 = 7;
pub const TYPE_PRIM_U64: u16 = 8;
pub const TYPE_PRIM_I64: u16 = 9;
pub const TYPE_PRIM_F32: u16 = 10;
pub const TYPE_PRIM_F64: u16 = 11;

pub const TYPE_DIM_MASK: u16 = 0x0030;
pub const TYPE_DIM_SCALAR: u16 = 0 << 4;
pub const TYPE_DIM_1D: u16 = 1 << 4;
pub const TYPE_DIM_2D: u16 = 2 << 4;
pub const TYPE_DIM_3D: u16 = 3 << 4;

pub const TYPE_HINT_MASK: u16 = 0x3f00;

/// Data is array of primitive without further hint.
///
/// * Any dimensions are allowed. No ghost dimensions are allowed.
///
/// * Any primitive is allowed.
pub const TYPE_HINT_NONE: u16 = 0 << 8;

/// Data is plain text.
///
/// * Dimensions must be `TYPE_DIM_1D` with the shape.X equal to the number of (byte) chars.
///   No ghost dimensions are allowed and must be zero.
///
/// * Primitive can be `TYPE_PRIM_U8`, `TYPE_PRIM_I8` and must be valid UTF-8 encoded text.
///
/// * Primitive can be `TYPE_PRIM_U16` and must be valid UTF-16 (little endian) encoded text.
///
/// Note that nul chars are allowed as per unicode standard.
pub const TYPE_HINT_TEXT: u16 = 1 << 8;

/// Data is JSON text.
///
/// * Dimension can be `TYPE_DIM_SCALAR` with no ghost dimensions and must be zero.
///
/// * Dimension can be `TYPE_DIM_1D`. This means the JSON must be an array with length equal to shape.X.
///   No ghost dimensions are allowed and must be zero.
///
/// * Primitive must be `TYPE_PRIM_CUSTOM`.
///
/// Data must be UTF-8 encoded text and be valid JSON.
/// Shape is related to the JSON structure, not the text.
pub const TYPE_HINT_JSON: u16 = 2 << 8;

/// Data is file offsets to other datasets.
///
/// * Dimensions can be `TYPE_DIM_SCALAR`, `TYPE_DIM_1D` or `TYPE_DIM_2D`.
///   Must have a single ghost dimension equal to 2.
///
/// * Primitive must be `TYPE_PRIM_U64`.
pub const TYPE_HINT_DATASET: u16 = 3 << 8;

/// Data is an index into another table.
///
/// Must have the `index_name` set pointing to a valid table.
/// Either a sibling (within the same dataset) or parent dataset.
/// The target table must have dimension `TYPE_DIM_1D`.
///
/// * Dimension can be anything, including any ghost dimensions.
///
/// * Primitive must be one of `U8`, `I8`, `U16`, `I16`, `U32`, `I32`, `U64`, `I64`.
///
/// All values in this table must be less than the length of the target table.
/// For signed primitives negative values are not allowed.
pub const TYPE_HINT_INDEX: u16 = 4 << 8;

/// Data is pairs of `[start, end)` indices into another table.
///
/// Must have the `index_name` set pointing to a valid table.
/// Either a sibling (within the same dataset) or parent dataset.
/// The target table must have dimension `TYPE_DIM_1D`.
///
/// * Dimensions can be anything except `TYPE_DIM_3D`.
///   Must have a single ghost dimension of length 2.
///
/// * Primitive must be one of `U8`, `I8`, `U16`, `I16`, `U32`, `I32`, `U64`, `I64`.
///
/// All values in this table must be a valid range in the target table.
/// For signed primitives negative values are not allowed.
pub const TYPE_HINT_RANGE: u16 = 5 << 8;
pub const TYPE_HINT_COORD: u16 = 6 << 8;
pub const TYPE_HINT_HATCH: u16 = 7 << 8;
pub const TYPE_HINT_TRANSFORM: u16 = 8 << 8;
pub const TYPE_HINT_RGB: u16 = 9 << 8;

pub const T_FILE_OFFSET: u16 = TYPE_HINT_DATASET | TYPE_DIM_1D | TYPE_PRIM_U64;

pub const fn type_prim_align(type_info: u16) -> usize {
	match type_info & TYPE_PRIM_MASK {
		TYPE_PRIM_U16 | TYPE_PRIM_I16 => 2,
		TYPE_PRIM_U32 | TYPE_PRIM_I32 | TYPE_PRIM_F32 => 4,
		TYPE_PRIM_U64 | TYPE_PRIM_I64 | TYPE_PRIM_F64 => 8,
		_ => 1,
	}
}

pub const COMPRESS_NONE: u16 = 0;

// Family of simple compression schemes
pub const COMPRESS_SIMPLE_U16: u16 = 16 + 0;
pub const COMPRESS_SIMPLE_U32: u16 = 16 + 1;
pub const COMPRESS_SIMPLE_U64: u16 = 16 + 2;
pub const COMPRESS_SIMPLE_F32: u16 = 16 + 3;
pub const COMPRESS_SIMPLE_F64: u16 = 16 + 4;

#[derive(Copy, Clone, Default, dataview::Pod)]
#[repr(C)]
pub struct LookupEntry {
	pub hash: u32,
	pub offset: u16,
	pub len: u16,
}


#[derive(Copy, Clone, Default, dataview::Pod)]
#[repr(transparent)]
pub struct IndexU32(pub u32);

#[derive(Copy, Clone, Default, dataview::Pod)]
#[repr(C)]
pub struct Index2U32(pub u32, pub u32);

#[derive(Copy, Clone, Default, dataview::Pod)]
#[repr(C)]
pub struct Index3U32(pub u32, pub u32, pub u32);

#[derive(Copy, Clone, Default, dataview::Pod)]
#[repr(C)]
pub struct RangeU32 {
	pub start: u32,
	pub end: u32,
}

#[derive(Copy, Clone, Default, dataview::Pod)]
#[repr(C)]
pub struct Coord2F32 {
	pub x: f32,
	pub y: f32,
}

#[derive(Copy, Clone, Default, dataview::Pod)]
#[repr(C)]
pub struct Coord3F32 {
	pub x: f32,
	pub y: f32,
	pub z: f32,
}

#[derive(Copy, Clone, Default, dataview::Pod)]
#[repr(C)]
pub struct HatchF32 {
	pub x1: f32,
	pub y1: f32,
	pub x2: f32,
	pub y2: f32,
}

#[derive(Copy, Clone, Default, dataview::Pod)]
#[repr(C)]
pub struct Transform2F32 {
	pub a11: f32,
	pub a12: f32,
	pub a13: f32,
	pub a21: f32,
	pub a22: f32,
	pub a23: f32,
}

#[derive(Copy, Clone, Default, dataview::Pod)]
#[repr(C)]
pub struct Transform3F32 {
	pub a11: f32,
	pub a12: f32,
	pub a13: f32,
	pub a14: f32,
	pub a21: f32,
	pub a22: f32,
	pub a23: f32,
	pub a24: f32,
	pub a31: f32,
	pub a32: f32,
	pub a33: f32,
	pub a34: f32,
}
