
#[derive(Debug)]
pub enum ParseError {
	Int(std::num::ParseIntError),
	Float(std::num::ParseFloatError),
}
impl std::error::Error for ParseError {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			ParseError::Int(err) => Some(err),
			ParseError::Float(err) => Some(err),
		}
	}
}
impl std::fmt::Display for ParseError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			ParseError::Int(err) => err.fmt(f),
			ParseError::Float(err) => err.fmt(f),
		}
	}
}

pub trait ParseNum: Sized {
	fn parse(s: &str) -> Result<Self, ParseError>;
}

macro_rules! parse_int {
	($($ty:ty),*) => {
		$(impl ParseNum for $ty {
			fn parse(mut s: &str) -> Result<Self, ParseError> {
				let radix;
				if s.starts_with("0x") {
					s = &s[2..];
					radix = 16;
				}
				else {
					radix = 10;
				}
				<$ty>::from_str_radix(s, radix)
					.map_err(|err| ParseError::Int(err))
			}
		})*
	};
}

macro_rules! parse_float {
	($($ty:ty),*) => {
		$(impl ParseNum for $ty {
			fn parse(s: &str) -> Result<Self, ParseError> {
				<$ty as std::str::FromStr>::from_str(s)
					.map_err(|err| ParseError::Float(err))
			}
		})*
	};
}

parse_int!(u8, u16, u32, u64, i8, i16, i32, i64);
parse_float!(f32, f64);

// Squash unnecessary characters to spaces
pub fn preprocess(text: &mut String) {
	unsafe {
		let text = text.as_mut_vec();
		for chr in text {
			let is_important =
				*chr >= b'a' && *chr <= b'z' ||
				*chr >= b'A' && *chr <= b'Z' ||
				*chr >= b'0' && *chr <= b'9' ||
				*chr == b'.' ||
				*chr == b'-' ||
				*chr == b'+';
			if !is_important {
				*chr = b' ';
			}
		}
	}
}

pub fn parse_all<T: ParseNum>(text: &str) -> Vec<T> {
	let mut data = Vec::new();
	for snum in text.split_ascii_whitespace() {
		let val = expect!(T::parse(snum), "Parse error "{snum:?});
		data.push(val);
	}
	return data;
}
