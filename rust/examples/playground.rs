use udf::AsDataRef;
use dataview::PodMethods;

fn main() {
	let mut writer = udf::FileIO::create("sample.udf", [0; 4]).unwrap();

	let mut ds = udf::Dataset::new();
	for &name in &["Floats", "Texts"] {
		ds.names.add(name, udf::hash(name));
	}
	ds.add_table(udf::TableRef {
		key_name: udf::hash("Floats"),
		data: FLOATS[..].as_data_ref(),
		..Default::default()
	});

	let text_fo = writer.add_dataset(&create_texts().finalize()).unwrap();
	ds.add_table(udf::TableRef {
		key_name: udf::hash("Texts"),
		data: text_fo.as_data_ref(),
		..Default::default()
	});

	let fo = writer.add_dataset(&ds.finalize()).unwrap();
	writer.set_root(fo);
	writer.write_header().unwrap();
}

static FLOATS: [f32; 6] = [0.0, 1.0, 2.0, 3.0, 3.141592, 42.0];


fn create_texts() -> udf::Dataset {
	let s = concat!(
		"Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.\n",
		"Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.\n",
		"Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur.\n",
		"Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.");

	const TEXT_LIST: [&str; 10] = [
		"I'm going to the (store) to pick up some groceries.",
		"She has a lot of (experience) in her field and is very knowledgeable.",
		"He always (wears) a suit to work because he thinks it looks professional.",
		"The (dog) barked at the mailman and scared him away.",
		"We need to (book) the tickets for the concert as soon as possible.",
		"The (book) was very interesting and I couldn't put it down.",
		"I can't believe it's already (summer) and the weather is so hot.",
		"The (mountains) are a beautiful sight to see when you're on a hike.",
		"She (speaks) three different languages fluently.",
		"He (works) as a software developer at a tech company.",
	];

	let mut ds = udf::Dataset::new();
	for &name in &["Text UTF-8", "Text UTF-16", "Text UTF-32", "List UTF-8", "List UTF-16", "List UTF-32"] {
		ds.names.add(name, udf::hash(name));
	}

	ds.add_table(udf::TableRef {
		key_name: udf::hash!("Text UTF-8"),
		data: udf::DataRef {
			bytes: s.as_bytes(),
			compress_info: 0,
			shape: [s.len() as u32, 0],
			type_info: udf::format::TYPE_PRIM_U8 | udf::format::TYPE_DIM_SCALAR | udf::format::TYPE_HINT_TEXT,
		},
		..Default::default()
	});

	let utf16 = s.encode_utf16().collect::<Vec<u16>>();
	ds.add_table(udf::TableRef {
		key_name: udf::hash!("Text UTF-16"),
		data: udf::DataRef {
			bytes: utf16.as_bytes(),
			compress_info: 0,
			shape: [utf16.len() as u32, 0],
			type_info: udf::format::TYPE_PRIM_U16 | udf::format::TYPE_DIM_SCALAR | udf::format::TYPE_HINT_TEXT,
		},
		..Default::default()
	});

	let utf32 = s.chars().map(|c| c as u32).collect::<Vec<u32>>();
	ds.add_table(udf::TableRef {
		key_name: udf::hash!("Text UTF-32"),
		data: udf::DataRef {
			bytes: utf32.as_bytes(),
			compress_info: 0,
			shape: [utf32.len() as u32, 0],
			type_info: udf::format::TYPE_PRIM_U32 | udf::format::TYPE_DIM_SCALAR | udf::format::TYPE_HINT_TEXT,
		},
		..Default::default()
	});

	let slist8 = encode_text_list(&TEXT_LIST);
	ds.add_table(udf::TableRef {
		key_name: udf::hash!("List UTF-8"),
		data: udf::DataRef {
			bytes: slist8.0.as_bytes(),
			compress_info: 0,
			shape: [TEXT_LIST.len() as u32, slist8.1 as u32],
			type_info: udf::format::TYPE_PRIM_U8 | udf::format::TYPE_DIM_1D | udf::format::TYPE_HINT_TEXT,
		},
		..Default::default()
	});

	let slist16 = slist8.0.encode_utf16().collect::<Vec<u16>>();
	ds.add_table(udf::TableRef {
		key_name: udf::hash!("List UTF-16"),
		data: udf::DataRef {
			bytes: slist16.as_bytes(),
			compress_info: 0,
			shape: [TEXT_LIST.len() as u32, slist8.1 as u32],
			type_info: udf::format::TYPE_PRIM_U16 | udf::format::TYPE_DIM_1D | udf::format::TYPE_HINT_TEXT,
		},
		..Default::default()
	});

	let slist32 = slist8.0.chars().map(|c| c as u32).collect::<Vec<u32>>();
	ds.add_table(udf::TableRef {
		key_name: udf::hash!("List UTF-32"),
		data: udf::DataRef {
			bytes: slist32.as_bytes(),
			compress_info: 0,
			shape: [TEXT_LIST.len() as u32, slist8.1 as u32],
			type_info: udf::format::TYPE_PRIM_U32 | udf::format::TYPE_DIM_1D | udf::format::TYPE_HINT_TEXT,
		},
		..Default::default()
	});

	return ds;
}

fn encode_text_list(strings: &[&str]) -> (String, usize) {
	let width = strings.iter().map(|s| s.len()).max().unwrap();
	let mut buffer = String::with_capacity(strings.len() * width);
	for &s in strings {
		buffer.push_str(s);
		for _ in s.len()..width {
			buffer.push_str("\0");
		}
	}
	return (buffer, width);
}
