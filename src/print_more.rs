use std::fmt;
use crate::*;

/// File size formatter.
///
/// ```
/// assert_eq!(udf::FileSize(0).to_string(), "0 bytes");
/// assert_eq!(udf::FileSize(1).to_string(), "1 byte");
/// assert_eq!(udf::FileSize(1023).to_string(), "1023 bytes");
/// assert_eq!(udf::FileSize(5869).to_string(), "5.73 KiB");
/// assert_eq!(udf::FileSize(41190000).to_string(), "39.28 MiB");
/// ```
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, dataview::Pod)]
#[repr(transparent)]
pub struct FileSize(pub u64);

impl fmt::Display for FileSize {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		if self.0 < 1024 {
			let unit = if self.0 == 1 { " byte" } else { " bytes" };
			write!(f, "{}{}", self.0, unit)
		}
		else {
			let (size, unit);
			if self.0 < 1024 * 1024 {
				size = self.0 as f64 / 1024.0;
				unit = " KiB";
			}
			else if self.0 < 1024 * 1024 * 1024 {
				size = self.0 as f64 / (1024.0 * 1024.0);
				unit = " MiB";
			}
			else if self.0 < 1024 * 1024 * 1024 * 1024 {
				size = self.0 as f64 / (1024.0 * 1024.0 * 1024.0);
				unit = " GiB";
			}
			else {
				size = self.0 as f64 / (1024.0 * 1024.0 * 1024.0 * 1024.0);
				unit = " TiB";
			}
			write!(f, "{:.2}{}", size, unit)
		}
	}
}

/// Identifier formatter.
///
/// ```
/// assert_eq!(udf::PrintId([0; 4]).to_string(), "");
/// assert_eq!(udf::PrintId(*b"UDF0").to_string(), "UDF0");
/// assert_eq!(udf::PrintId(*b"abc\0").to_string(), "abc");
/// assert_eq!(udf::PrintId(*b"z\0\0\0").to_string(), "z");
/// assert_eq!(udf::PrintId(*b"\0\0f\0").to_string(), "\\0\\0f");
/// ```
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, dataview::Pod)]
#[repr(transparent)]
pub struct PrintId(pub [u8; 4]);

impl fmt::Display for PrintId {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		use std::fmt::Write;

		// Shorten the code if terminated with nul bytes
		let mut bytes = self.0.as_slice();
		while bytes.last() == Some(&b'\0') {
			bytes = &bytes[..bytes.len() - 1];
		}

		for &byte in bytes {
			if byte == 0 {
				f.write_str("\\0")?;
			}
			else if byte >= b' ' && byte < 0x7f {
				f.write_char(byte as char)?;
			}
			else {
				write!(f, "\\x{:02x}", byte)?;
			}
		}
		Ok(())
	}
}

/// Type info formatter.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, dataview::Pod)]
#[repr(transparent)]
pub struct PrintTypeInfo(pub u16);

#[doc(hidden)]
impl PrintTypeInfo {
	pub const fn prim(type_info: u16) -> Option<&'static str> {
		match type_info & format::TYPE_PRIM_MASK {
			format::TYPE_PRIM_CUSTOM => Some("custom"),
			format::TYPE_PRIM_BIT => Some("bit"),
			format::TYPE_PRIM_U8 => Some("u8"),
			format::TYPE_PRIM_I8 => Some("i8"),
			format::TYPE_PRIM_U16 => Some("u16"),
			format::TYPE_PRIM_I16 => Some("i16"),
			format::TYPE_PRIM_U32 => Some("u32"),
			format::TYPE_PRIM_I32 => Some("i32"),
			format::TYPE_PRIM_U64 => Some("u64"),
			format::TYPE_PRIM_I64 => Some("i64"),
			format::TYPE_PRIM_BFLOAT16 => Some("bfloat16"),
			format::TYPE_PRIM_F32 => Some("f32"),
			format::TYPE_PRIM_F64 => Some("f64"),
			format::TYPE_PRIM_DECIMAL => Some("decimal"),
			_ => None,
		}
	}
	pub const fn dim(type_info: u16) -> Option<&'static str> {
		match type_info & format::TYPE_DIM_MASK {
			format::TYPE_DIM_SCALAR => Some("scalar"),
			format::TYPE_DIM_1D => Some("1d"),
			format::TYPE_DIM_2D => Some("2d"),
			format::TYPE_DIM_3D => Some("3d"),
			_ => None,
		}
	}
	pub const fn hint(type_info: u16) -> Option<&'static str> {
		match type_info & format::TYPE_HINT_MASK {
			format::TYPE_HINT_NONE => Some("none"),
			format::TYPE_HINT_TEXT => Some("text"),
			format::TYPE_HINT_JSON => Some("json"),
			format::TYPE_HINT_DATASET => Some("dataset"),
			format::TYPE_HINT_XDATASET => Some("xdataset"),
			format::TYPE_HINT_INDEX => Some("index"),
			format::TYPE_HINT_RANGE => Some("range"),
			format::TYPE_HINT_COORD => Some("coord"),
			format::TYPE_HINT_HATCH => Some("line"),
			format::TYPE_HINT_TRANSFORM => Some("transform"),
			format::TYPE_HINT_COLOR => Some("color"),
			format::TYPE_HINT_TIME => Some("time"),
			format::TYPE_HINT_UTS => Some("uts"),
			format::TYPE_HINT_GUID => Some("guid"),
			format::TYPE_HINT_FILE => Some("file"),
			_ => None,
		}
	}

}

impl fmt::Display for PrintTypeInfo {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let type_info = self.0;
		write!(f, "{:x?} {} {} {}",
			type_info,
			Self::hint(type_info).unwrap_or("?"),
			Self::dim(type_info).unwrap_or("?"),
			Self::prim(type_info).unwrap_or("?"),
		)
	}
}
