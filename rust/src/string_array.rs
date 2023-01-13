use super::*;

/// 
pub fn build_string_array_utf8<'a, 't, T: Clone + IntoIterator<Item = &'t str>>(strings: T, data: &'a mut Vec<u8>) -> DataRef<'a> {
	// First find the max length
	let width = strings.clone().into_iter().map(|s| s.len()).max().unwrap_or(0);
	// Add the strings to the data
	let mut new_len = 0;
	let mut rows = 0;
	for s in strings {
		data.extend_from_slice(s.as_bytes());
		new_len += width;
		data.resize(new_len, b'\0');
		rows += 1;
	}
	DataRef {
		type_info: format::TYPE_PRIM_U8 | format::TYPE_DIM_1D | format::TYPE_HINT_TEXT,
		shape: Shape::D2(rows as u32, width as u32),
		bytes: data.as_slice(),
		..Default::default()
	}
}
