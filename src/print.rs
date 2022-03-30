use std::fmt;

use crate::*;

/// NdArray formatter.
///
/// Formats the elements as numpy would print multidimensional arrays.
#[derive(Clone, Debug)]
pub struct PrintArray {
	strings: String, // Buffer of all the formatted elements
	indices: Vec<u32>, // Indices into the strings buffer, one per element
	element_width: u32, // Max element width, used to align when printing
	line_width: u32, // Number of chars per line for the purpose of inserting newlines
	shape: Shape, // Shape of the data, must match the number of indices
}

impl PrintArray {
	/// Creates a new instance.
	#[inline]
	pub const fn new(shape: Shape) -> PrintArray {
		PrintArray { strings: String::new(), indices: Vec::new(), element_width: 0, line_width: 75, shape }
	}

	/// Reserve capacity for the formatted elements.
	#[inline]
	pub fn reserve(&mut self, len: usize, avg_size: usize) {
		self.indices.reserve(len);
		self.strings.reserve(len * avg_size);
	}

	/// Returns the configured line width.
	#[inline]
	pub fn line_width(&self) -> u32 {
		self.line_width
	}

	/// Sets the line width for the purpose of inserting line breaks (default 75).
	///
	/// If the element width is less than the line width no line breaks will be inserted.
	///
	/// Set to zero to print all elements on a single line without line breaks.
	#[inline]
	pub fn set_line_width(&mut self, line_width: u32) -> &mut PrintArray {
		self.line_width = line_width;
		return self;
	}

	/// Returns the shape of the elements.
	#[inline]
	pub fn shape(&self) -> Shape {
		self.shape
	}

	/// Sets the shape of the elements.
	///
	/// Formatting panics if the length of the shape does not match the number of elements.
	#[inline]
	pub fn set_shape(&mut self, shape: Shape) -> &mut PrintArray {
		self.shape = shape;
		return self;
	}

	/// Adds a string element to print.
	pub fn push_str(&mut self, string: &str) -> fmt::Result {
		let start = self.strings.len();
		self.strings.push_str(string);
		let len = self.strings.len() - start;

		if start >= 0x1000000 || len >= 0x100 {
			return Err(fmt::Error);
		}
		self.element_width = u32::max(self.element_width, len as u32);
		self.indices.push((start << 8 | len) as u32);

		Ok(())
	}

	/// Adds a formattable element to print.
	pub fn push_fmt(&mut self, args: fmt::Arguments) -> fmt::Result {
		let start = self.strings.len();
		<String as fmt::Write>::write_fmt(&mut self.strings, args)?;
		let len = self.strings.len() - start;

		if start >= 0x1000000 || len >= 0x100 {
			return Err(fmt::Error);
		}
		self.element_width = u32::max(self.element_width, len as u32);
		self.indices.push((start << 8 | len) as u32);
		Ok(())
	}
}

impl fmt::Display for PrintArray {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		assert_eq!(self.shape.len(), self.indices.len(), "number of elements must match the length of the shape!");

		match self.shape {
			Shape::Scalar => {
				write!(f, "{}", self.strings)?;
			},
			Shape::D1(x) => {
				print0(self, f, 0, x as usize, 1)?;
			},
			Shape::D2(x, y) => {
				print1(self, f, 0, x as usize, y as usize, 1)?;
			},
			Shape::D3(x, y, z) => {
				let x = x as usize;
				let y = y as usize;
				let z = z as usize;
				let stride = y * z;

				f.write_str("[")?;

				let line_break = if self.line_width == 0 { ", " } else { ",\n\n " };

				let mut comma = false;
				for xi in 0..x {
					if comma {
						f.write_str(line_break)?;
					}
					comma = true;

					let start = xi * stride;
					print1(self, f, start, y, z, 2)?;
				}

				f.write_str("]")?;
			},
		}

		Ok(())
	}
}

fn print0(this: &PrintArray, f: &mut fmt::Formatter, start: usize, end: usize, indent: usize) -> fmt::Result {
	f.write_str("[")?;

	let line_break = &",\n      "[..2 + indent];
	let width = this.element_width as usize;
	let cols = this.line_width / this.element_width;

	let mut column = 0;
	let mut comma = false;
	for xi in start..end {
		if comma {
			if cols > 0 && column == 0 {
				f.write_str(line_break)?;
			}
			else {
				f.write_str(", ")?;
			}
		}
		comma = true;

		if cols > 0 {
			column += 1;
			if column == cols {
				column = 0;
			}
		}

		let index = this.indices[xi];
		let start = (index >> 8) as usize;
		let end = start + (index & 0xff) as usize;
		let value = &this.strings[start..end];

		if cols > 0 {
			write!(f, "{: >1$}", value, width)?;
		}
		else {
			write!(f, "{}", value)?;
		}
	}

	f.write_str("]")?;
	Ok(())
}

fn print1(this: &PrintArray, f: &mut fmt::Formatter, start: usize, rows: usize, cols: usize, indent: usize) -> fmt::Result {
	f.write_str("[")?;

	let line_break = if this.line_width == 0 { ", " } else { &",\n      "[..2 + indent] };

	let mut comma = false;
	for row in 0..rows {
		if comma {
			f.write_str(line_break)?;
		}
		comma = true;

		let start = start + row * cols;
		let end = start + cols;
		print0(this, f, start, end, indent + 1)?;
	}

	f.write_str("]")?;
	Ok(())
}
