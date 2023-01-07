use std::{fmt, str};
use crate::*;

/// Shape.
///
/// Up to a maximum of three dimensions.
///
/// Due to how the shape is encoded the range of each dimension is restricted:
///
/// * 32 bits for the first axis
/// * 24 bits for the second axis
/// * 8 bits for the third axis
///
/// The total number of elements cannot exceed 2^64.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Shape {
	/// Scalar
	Scalar,
	/// 1D
	D1(u32),
	/// 2D
	///
	/// Reserves 24 bits for the second axis.
	D2(u32, u32),
	/// 3D
	///
	/// Reserves 24 bits for second axis and 8 bits for the third axis.
	D3(u32, u32, u8),
}

impl Shape {
	#[inline]
	pub fn from_type_info(type_info: u16, shape: [u32; 2]) -> Shape {
		match type_info & format::TYPE_DIM_MASK {
			format::TYPE_DIM_SCALAR => Shape::Scalar,
			format::TYPE_DIM_1D => Shape::D1(shape[0]),
			format::TYPE_DIM_2D => Shape::D2(shape[0], shape[1] & 0xffffff),
			format::TYPE_DIM_3D => Shape::D3(shape[0], shape[1] & 0xffffff, (shape[1] >> 24) as u8),
			_ => unreachable!()
		}
	}

	#[inline]
	pub fn from_shape(type_info: u16, shape: [u32; 2]) -> Shape {
		let type_dims = type_info & format::TYPE_DIM_MASK;
		let x = shape[0];
		let y = shape[1] & 0xffffff;
		let z = (shape[1] >> 24) as u8;
		if z == 0 && type_dims < format::TYPE_DIM_3D {
			if y == 0 && type_dims < format::TYPE_DIM_2D {
				if x == 0 && type_dims < format::TYPE_DIM_1D {
					return Shape::Scalar;
				}
				return Shape::D1(x);
			}
			return Shape::D2(x, y);
		}
		return Shape::D3(x, y, z);
	}

	/// Returns the total number of elements.
	#[inline]
	pub fn len(&self) -> usize {
		match self {
			&Shape::Scalar => 1,
			&Shape::D1(x) => x as usize,
			&Shape::D2(x, y) => x as usize * y as usize,
			&Shape::D3(x, y, z) => x as usize * y as usize * z as usize,
		}
	}

	/// Reshapes as a 1D array.
	#[inline]
	pub fn flatten(&self) -> Shape {
		Shape::D1(self.len() as u32)
	}

	/// Encodes the shape.
	pub fn encode(&self) -> (u16, [u32; 2]) {
		match self {
			&Shape::Scalar => (format::TYPE_DIM_SCALAR, [0, 0]),
			&Shape::D1(x) => (format::TYPE_DIM_1D, [x, 0]),
			&Shape::D2(x, y) => (format::TYPE_DIM_2D, [x, y & 0xffffff]),
			&Shape::D3(x, y, z) => (format::TYPE_DIM_3D, [x, y & 0xffffff | (z as u32) << 24]),
		}
	}
}

impl fmt::Display for Shape {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Shape::Scalar => f.write_str("scalar"),
			Shape::D1(x) => write!(f, "{}", x),
			Shape::D2(x, y) => write!(f, "{}x{}", x, y),
			Shape::D3(x, y, z) => write!(f, "{}x{}x{}", x, y, z),
		}
	}
}

impl str::FromStr for Shape {
	type Err = ParseError;
	fn from_str(string: &str) -> Result<Self, ParseError> {
		if string == "scalar" {
			return Ok(Shape::Scalar);
		}
		let mut split = string.split("x");
		let x = match split.next() {
			Some(x) => x.parse::<u32>()?,
			None => return Err(ParseError::InvalidFormat),
		};
		let y = match split.next() {
			Some(y) => y.parse::<u32>()?,
			None => return Ok(Shape::D1(x)),
		};
		if y >= (1 << 24) {
			return Err(ParseError::Overflow);
		}
		let z = match split.next() {
			Some(z) => z.parse::<u32>()?,
			None => return Ok(Shape::D2(x, y)),
		};
		if z >= (1 << 8) {
			return Err(ParseError::Overflow);
		}
		match split.next() {
			Some(_) => Err(ParseError::InvalidFormat),
			None => Ok(Shape::D3(x, y, z as u8)),
		}
	}
}
