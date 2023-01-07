use std::{fmt, mem};

// This file exists to print floats in a format nicer for consumption
// Rust's standard float formatter does not print the decimal separator (followed by `0`) if it's an integer
// This makes it so much more annoying to copy the resulting values back into source code
// PrintF32 and PrintF64 amend this annoyance by appending `.0` where appropriate
// PrintArray code uses this code to format floats

struct PrintFloat<'a, T: ?Sized> {
	inner: &'a mut T,
	seen_dot: bool,
}

impl<'a, T> From<&'a mut T> for PrintFloat<'a, T> {
	#[inline]
	fn from(inner: &'a mut T) -> Self {
		PrintFloat { inner, seen_dot: false }
	}
}

impl<'a, T: ?Sized + fmt::Write> fmt::Write for PrintFloat<'a, T> {
	fn write_str(&mut self, s: &str) -> fmt::Result {
		// This code makes assumptions about how Rust's standard formatting for floats behaves
		// Decimal separator and scientific notation pass `"."` and `"e"` as separate strings to the writer
		if s.len() == 1 {
			let b = s.as_bytes();
			if b[0] == b'.' {
				self.seen_dot = true;
			}
			else if b[0] == b'e' {
				if !self.seen_dot {
					self.inner.write_str(".0")?;
				}
				self.seen_dot = true;
			}
		}
		// This branch is used when formatting a number < 1.0
		else if s.len() == 2 {
			let b = s.as_bytes();
			if b[0] == b'0' && b[1] == b'.' {
				self.seen_dot = true;
			}
		}
		// write!(self.inner, "{{{}}}", s)
		self.inner.write_str(s)
	}
}


#[repr(transparent)]
pub struct PrintF32(f32);

impl PrintF32 {
	#[inline]
	pub fn wrap(v: &f32) -> &PrintF32 {
		unsafe { mem::transmute(v) }
	}
}

impl fmt::Display for PrintF32 {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut wrapf = PrintFloat { inner: f, seen_dot: false };
		use std::fmt::Write;
		write!(wrapf, "{}", self.0)?;
		if !wrapf.seen_dot {
			f.write_str(".0")?;
		}
		Ok(())
	}
}

#[repr(transparent)]
pub struct PrintF64(f64);

impl PrintF64 {
	#[inline]
	pub fn wrap(v: &f64) -> &PrintF64 {
		unsafe { mem::transmute(v) }
	}
}

impl fmt::Display for PrintF64 {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut wrapf = PrintFloat { inner: f, seen_dot: false };
		use std::fmt::Write;
		write!(wrapf, "{}", self.0)?;
		if !wrapf.seen_dot {
			f.write_str(".0")?;
		}
		Ok(())
	}
}
