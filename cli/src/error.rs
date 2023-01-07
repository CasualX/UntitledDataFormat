use std::{error, fmt};

#[derive(Clone, Debug)]
pub struct StringError {
	string: String,
}

impl<'a> From<&'a str> for StringError {
	fn from(s: &'a str) -> Self {
		StringError { string: String::from(s) }
	}
}

impl fmt::Display for StringError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(&self.string)
	}
}

impl error::Error for StringError {}
