use std::str;
use crate::*;

#[derive(Copy, Clone, Debug)]
pub enum PathEl<'a> {
	Dir { name: &'a str, index: u32 },
	Name(&'a str),
}

impl<'a> PathEl<'a> {
	#[inline]
	pub fn name(&self) -> &'a str {
		match self {
			PathEl::Dir { name, .. } => name,
			PathEl::Name(name) => name,
		}
	}

	pub fn parse(state: &mut &'a str) -> Result<PathEl<'a>, ParseError> {
		if state.is_empty() {
			return Err(ParseError::InvalidFormat);
		}

		let mut string = *state;

		let mut i = 0;
		let mut j = None;
		loop {
			if i < string.len() {
				if string.as_bytes()[i] == b'[' {
					if j.is_some() {
						// Already found a `[` which is invalid
						return Err(ParseError::InvalidFormat);
					}
					j = Some(i);
				}
				if string.as_bytes()[i] == b'.' {
					*state = &string[i + 1..];
					string = &string[..i];
					break;
				}
				i += 1;
			}
			else {
				*state = &string[..0];
				break;
			}
		}

		if let Some(j) = j {
			if string.as_bytes().last() != Some(&b']') {
				return Err(ParseError::InvalidFormat);
			}
			let name = &string[..j];
			let index = &string[j + 1..string.len() - 1];
			let index = index.parse()?;
			return Ok(PathEl::Dir { name, index });
		}
		else {
			return Ok(PathEl::Name(string));
		}
	}
}
