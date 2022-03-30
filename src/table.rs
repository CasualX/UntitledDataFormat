use crate::*;

#[derive(Copy, Clone, Default)]
pub struct TableRef<'a> {
	pub key_name: u32,
	pub data: DataRef<'a>,
	pub index_name: u32,
	pub related_name: u32,
}

impl<'a> TableRef<'a> {
}
