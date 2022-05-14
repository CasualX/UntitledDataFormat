/*!
Disk format structs.
*/

use std::mem;

/// The UDF file header.
#[derive(Copy, Clone, Default, dataview::Pod)]
#[repr(C)]
pub struct Header {
	/// Magic file format identifier.
	///
	/// Must be set to [`MAGIC`](Self::MAGIC).
	pub magic: [u8; 4],
	/// Identifies the file format conventions.
	///
	/// This field can be used to quickly identify if this UDF file is deliberately created for your application.
	pub id: [u8; 4],
	/// Bump allocator, next offset.
	pub next: u64,
	/// The root dataset.
	pub root: FileOffset,
	pub temp: [u64; 4],
}

const _: [(); 0] = [(); mem::size_of::<Header>() % 16];

impl Header {
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
	/// Must be 8-byte aligned.
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
		self.offset & 0xf == 0 && self.size & 0x7 == 0
	}
}

#[derive(Copy, Clone, Debug, Default, dataview::Pod)]
#[repr(C)]
pub struct DatasetHeader {
	/// Check value allows checking if this is really a dataset.
	///
	/// Must be set to [`CHECK`](Self::CHECK).
	pub check: u32,
	/// CRC32 of the header (starting from the next field) and the tables.
	///
	/// If zero, the checksum is absent.
	pub csum: u32,
	/// CRC32 of the data storage.
	///
	/// If zero, the checksum is absent.
	pub storage_csum: u32,
	/// Number of tables following the header.
	pub len: u8,
	/// Padding, must be zero.
	pub pad_len: u8,
	/// Maximum number of tables following the header.
	///
	/// Ignore this field if less than `len`.
	pub max_len: u8,
	/// Padding, must be zero.
	pub pad_max_len: u8,
	/// Identifies the dataset convention.
	pub id: [u8; 4],
	/// Unused padding, must be zero.
	pub flags: u32,
}

const _: [(); 0] = [(); mem::size_of::<DatasetHeader>() % 8];

impl DatasetHeader {
	pub const CHECK: u32 = 0x7fcea59b;
}

#[derive(Copy, Clone, Debug, Default, dataview::Pod)]
#[repr(C)]
pub struct Table {
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
	/// Size of the data (after compression) in bytes.
	pub data_size: u32,
	/// Multidimensional shape of the data.
	pub data_shape: [u32; 2],
	/// Must be zero.
	pub pad: u32,
	/// When an index type hint is used this specifies which datatable the indices go into. Otherwise 0.
	///
	/// If the datatable by this name does not exist in the dataset, walk through parent datasets until found.
	pub index_name: u32,
	/// Struct of Arrays (SoA) indicates related datatable.
	pub related_name: u32,
}

// Datatable must be a multiple of 8 bytes to keep alignment easy to reason about
const _: [(); 0] = [(); mem::size_of::<Table>() % 8];

/// Special type for the names table.
pub const TYPE_NAMES: u16 = 0;

pub const TYPE_PRIM_MASK: u16 = 0x00cf;
pub const TYPE_PRIM_CUSTOM: u16 = 0;
pub const TYPE_PRIM_BIT: u16 = 1;
pub const TYPE_PRIM_U8: u16 = 2;
pub const TYPE_PRIM_I8: u16 = 3;
pub const TYPE_PRIM_U16: u16 = 4;
pub const TYPE_PRIM_I16: u16 = 5;
pub const TYPE_PRIM_U32: u16 = 6;
pub const TYPE_PRIM_I32: u16 = 7;
pub const TYPE_PRIM_U64: u16 = 8;
pub const TYPE_PRIM_I64: u16 = 9;
pub const TYPE_PRIM_BFLOAT16: u16 = 10;
pub const TYPE_PRIM_F32: u16 = 11;
pub const TYPE_PRIM_F64: u16 = 12;
pub const TYPE_PRIM_DECIMAL: u16 = 13;

pub const TYPE_DIM_MASK: u16 = 0x0030;
pub const TYPE_DIM_SCALAR: u16 = 0 << 4;
pub const TYPE_DIM_1D: u16 = 1 << 4;
pub const TYPE_DIM_2D: u16 = 2 << 4;
pub const TYPE_DIM_3D: u16 = 3 << 4;

pub const TYPE_HINT_MASK: u16 = 0xff00;

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
pub const TYPE_HINT_COLOR: u16 = 9 << 8;
pub const TYPE_HINT_TIME: u16 = 10 << 8;
pub const TYPE_HINT_UTS: u16 = 11 << 8;
pub const TYPE_HINT_GUID: u16 = 12 << 8;
pub const TYPE_HINT_FILE: u16 = 13 << 8;
pub const TYPE_HINT_CHAR_ARRAY: u16 = 14 << 8;

/// Reserved for file offsets to other encrypted datasets.
pub const TYPE_HINT_XDATASET: u16 = 15 << 8;

pub const T_FILE_OFFSET: u16 = TYPE_HINT_DATASET | TYPE_DIM_1D | TYPE_PRIM_U64;

pub const fn type_prim_align(type_info: u16) -> usize {
	match type_info & TYPE_PRIM_MASK {
		TYPE_PRIM_U16 | TYPE_PRIM_I16 | TYPE_PRIM_BFLOAT16 => 2,
		TYPE_PRIM_U32 | TYPE_PRIM_I32 | TYPE_PRIM_F32 | TYPE_PRIM_DECIMAL => 4,
		TYPE_PRIM_U64 | TYPE_PRIM_I64 | TYPE_PRIM_F64 => 8,
		_ => 1,
	}
}

pub const fn s_type_prim(type_info: u16) -> Option<&'static str> {
	match type_info & TYPE_PRIM_MASK {
		TYPE_PRIM_CUSTOM => Some("custom"),
		TYPE_PRIM_BIT => Some("bit"),
		TYPE_PRIM_U8 => Some("u8"),
		TYPE_PRIM_I8 => Some("i8"),
		TYPE_PRIM_U16 => Some("u16"),
		TYPE_PRIM_I16 => Some("i16"),
		TYPE_PRIM_U32 => Some("u32"),
		TYPE_PRIM_I32 => Some("i32"),
		TYPE_PRIM_U64 => Some("u64"),
		TYPE_PRIM_I64 => Some("i64"),
		TYPE_PRIM_BFLOAT16 => Some("bfloat16"),
		TYPE_PRIM_F32 => Some("f32"),
		TYPE_PRIM_F64 => Some("f64"),
		TYPE_PRIM_DECIMAL => Some("decimal"),
		_ => None,
	}
}
pub const fn s_type_dim(type_info: u16) -> Option<&'static str> {
	match type_info & TYPE_DIM_MASK {
		TYPE_DIM_SCALAR => Some("scalar"),
		TYPE_DIM_1D => Some("1d"),
		TYPE_DIM_2D => Some("2d"),
		TYPE_DIM_3D => Some("3d"),
		_ => None,
	}
}
pub const fn s_type_hint(type_info: u16) -> Option<&'static str> {
	match type_info & TYPE_HINT_MASK {
		TYPE_HINT_NONE => Some("none"),
		TYPE_HINT_TEXT => Some("text"),
		TYPE_HINT_JSON => Some("json"),
		TYPE_HINT_DATASET => Some("dataset"),
		TYPE_HINT_INDEX => Some("index"),
		TYPE_HINT_RANGE => Some("range"),
		TYPE_HINT_COORD => Some("point"),
		TYPE_HINT_HATCH => Some("line"),
		TYPE_HINT_TRANSFORM => Some("transform"),
		TYPE_HINT_COLOR => Some("color"),
		TYPE_HINT_TIME => Some("time"),
		TYPE_HINT_UTS => Some("uts"),
		TYPE_HINT_GUID => Some("guid"),
		_ => None,
	}
}

/*

16-bit, 32-bit and 64-bit integer compression scheme:

00 AAAAAA          delta from last value
01 BBBBBB BBBBBBBB delta from last value
10 CCCCCC          lookup value from index
110 DDDDD          repeat previous value (if not max repeats, lastv += 1)
111 EEEEE          uncompressed elements follow


00
01
10
1100
1101
1110
1111

*/

pub const COMPRESS_NONE: u16 = 0;

// Family of simple compression schemes
// pub const COMPRESS_SIMPLE_16_1: u16 = 8 + 0;
// pub const COMPRESS_SIMPLE_32_1: u16 = 8 + 1;
// pub const COMPRESS_SIMPLE_64_1: u16 = 8 + 2;
// pub const COMPRESS_SIMPLE_16_2: u16 = 8 + 3;
// pub const COMPRESS_SIMPLE_32_2: u16 = 8 + 4;
// pub const COMPRESS_SIMPLE_64_2: u16 = 8 + 5;
// pub const COMPRESS_SIMPLE_32_3: u16 = 8 + 6;
// pub const COMPRESS_SIMPLE_64_3: u16 = 8 + 7;

#[derive(Copy, Clone, Default, dataview::Pod)]
#[repr(C)]
pub struct NameDesc {
	pub hash: u32,
	pub start: u16,
	pub end: u16,
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
