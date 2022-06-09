use std::{error, fmt, num, str};
use crate::*;

/// Parse errors.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ParseError {
	InvalidFormat,
	OutOfBounds,
	Overflow,
	ParseIntError(num::ParseIntError),
}

impl From<num::ParseIntError> for ParseError {
	#[inline]
	fn from(err: num::ParseIntError) -> Self {
		ParseError::ParseIntError(err)
	}
}

impl fmt::Display for ParseError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			ParseError::InvalidFormat => f.pad("invalid format"),
			ParseError::OutOfBounds => f.pad("out of bounds"),
			ParseError::Overflow => f.pad("overflow"),
			ParseError::ParseIntError(err) => err.fmt(f),
		}
	}
}

impl error::Error for ParseError {
	fn source(&self) -> Option<&(dyn error::Error + 'static)> {
		match self {
			ParseError::ParseIntError(err) => Some(err),
			_ => None,
		}
	}
}

impl str::FromStr for format::FileOffset {
	type Err = ParseError;
	fn from_str(string: &str) -> Result<Self, Self::Err> {
		let mut split = string.split(":");
		let offset = match split.next() {
			Some(src) => parse_u64(src)?,
			None => return Err(ParseError::InvalidFormat),
		};
		let size = match split.next() {
			Some(src) => parse_u64(src)?,
			None => return Err(ParseError::InvalidFormat),
		};
		if split.next().is_some() {
			return Err(ParseError::InvalidFormat);
		}

		Ok(format::FileOffset { offset, size })
	}
}

fn parse_u64(mut src: &str) -> Result<u64, num::ParseIntError> {
	let radix = if src.starts_with("0x") { src = &src[2..]; 16 } else { 10 };
	u64::from_str_radix(src, radix)
}

impl fmt::Display for format::FileOffset {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{:#x}:{:#x}", self.offset, self.size)
	}
}
