"use strict"

export const TYPE_PRIM_MASK = 0x008f;
export const TYPE_PRIM_CUSTOM = 0;
export const TYPE_PRIM_U8 = 2;
export const TYPE_PRIM_I8 = 3;
export const TYPE_PRIM_U16 = 4;
export const TYPE_PRIM_I16 = 5;
export const TYPE_PRIM_U32 = 6;
export const TYPE_PRIM_I32 = 7;
export const TYPE_PRIM_U64 = 8;
export const TYPE_PRIM_I64 = 9;
export const TYPE_PRIM_F32 = 10;
export const TYPE_PRIM_F64 = 11;

export const TYPE_DIM_MASK = 0x0030;
export const TYPE_DIM_SCALAR = 0 << 4;
export const TYPE_DIM_1D = 1 << 4;
export const TYPE_DIM_2D = 2 << 4;
export const TYPE_DIM_3D = 3 << 4;

export const TYPE_HINT_MASK = 0x3f00;
export const TYPE_HINT_NONE = 0 << 8;
export const TYPE_HINT_TEXT = 1 << 8;
export const TYPE_HINT_JSON = 2 << 8;
export const TYPE_HINT_DATASET = 3 << 8;
export const TYPE_HINT_INDEX = 4 << 8;
export const TYPE_HINT_RANGE = 5 << 8;
export const TYPE_HINT_COORD = 6 << 8;
export const TYPE_HINT_LINE = 7 << 8;
export const TYPE_HINT_TRANSFORM = 8 << 8;
export const TYPE_HINT_RGB = 9 << 8;


let utf8Decoder = new TextDecoder('utf-8');

/**
 * 
 * @param {number} id
 * @returns string
 */
function decodeId(id) {
	if (id == 0) {
		return "{empty}";
	}
	let ids = String.fromCharCode(id & 0xff);
	if ((id & 0xffffff00) != 0) {
		ids += String.fromCharCode((id & 0xff00) >> 8);
		if ((id & 0xffff0000) != 0) {
			ids += String.fromCharCode((id & 0xff0000) >> 16);
			if ((id & 0xff000000) != 0) {
				ids += String.fromCharCode((id & 0xff000000) >> 24);
			}
		}
	}
	return ids;
}

/**
 * @param {number} type_info
 * @returns {{hint: number, dim: number, prim: number}}
 */
export function decodeTypeInfo(type_info) {
	let hint = type_info & TYPE_HINT_MASK;
	let dim = type_info & TYPE_DIM_MASK;
	let prim = type_info & TYPE_PRIM_MASK;
	return Object.freeze({ hint, dim, prim });
}

/**
 * @param {number} type_info
 * @param {number} data_shape0
 * @param {number} data_shape1
 * @returns number[]
 */
function decodeShape(type_info, data_shape0, data_shape1) {
	let type_dims = type_info & TYPE_DIM_MASK;
	let x = data_shape0;
	let y = data_shape1 & 0xffffff;
	let z = (data_shape1 >>> 24) & 0xff;
	if (z == 0 && type_dims < TYPE_DIM_3D) {
		if (y == 0 && type_dims < TYPE_DIM_2D) {
			if (x == 0 && type_dims < TYPE_DIM_1D) {
				return [];
			}
			return [x];
		}
		return [x, y];
	}
	return [x, y, z];
}

/**
 * @param {number} file_size
 * @returns string
 */
export function printFileSize(file_size) {
	if (typeof file_size != "number") {
		return "" + file_size;
	}
	if (file_size < 0) {
		return "-" + printFileSize(-file_size);
	}
	if (file_size < 1024) {
		let suffix = file_size == 1 ? " byte" : " bytes";
		return `${file_size}${suffix}`;
	}
	else {
		let size, unit;
		if (file_size < 1024 * 1024) {
			size = file_size / 1024.0;
			unit = " KiB";
		}
		else if (file_size < 1024 * 1024 * 1024) {
			size = file_size / (1024.0 * 1024.0);
			unit = " MiB";
		}
		else if (file_size < 1024 * 1024 * 1024 * 1024) {
			size = file_size / (1024.0 * 1024.0 * 1024.0);
			unit = " GiB";
		}
		else/* if (bytes < 1024 * 1024 * 1024 * 1024 * 1024)*/ {
			size = file_size / (1024.0 * 1024.0 * 1024.0 * 1024.0);
			unit = " TiB";
		}
		return `${size.toFixed(2)}${unit}`;
	}
}

/**
 * @param {{offset: number, size: number}} file_offset
 * @returns string
 */
export function printFileOffset(file_offset) {
	return `0x${file_offset.offset.toString(16)}:0x${file_offset.size.toString(16)}`;
}

/**
 * @param {number[]} shape
 */
export function printShape(shape) {
	return shape.join("x") || "scalar";
}

const tylookup_hint = {
	[TYPE_HINT_NONE]: "none",
	[TYPE_HINT_TEXT]: "text",
	[TYPE_HINT_JSON]: "json",
	[TYPE_HINT_DATASET]: "dataset",
	[TYPE_HINT_INDEX]: "index",
	[TYPE_HINT_RANGE]: "range",
	[TYPE_HINT_COORD]: "coord",
	[TYPE_HINT_LINE]: "line",
	[TYPE_HINT_TRANSFORM]: "transform",
};
const tylookup_dim = {
	[TYPE_DIM_SCALAR]: "scalar",
	[TYPE_DIM_1D]: "1d",
	[TYPE_DIM_2D]: "2d",
	[TYPE_DIM_3D]: "3d",
};
const tylookup_prim = {
	[TYPE_PRIM_CUSTOM]: "?",
	[TYPE_PRIM_U8]: "u8",
	[TYPE_PRIM_I8]: "i8",
	[TYPE_PRIM_U16]: "u16",
	[TYPE_PRIM_I16]: "i16",
	[TYPE_PRIM_U32]: "u32",
	[TYPE_PRIM_I32]: "i32",
	[TYPE_PRIM_U64]: "u64",
	[TYPE_PRIM_I64]: "i64",
	[TYPE_PRIM_F32]: "f32",
	[TYPE_PRIM_F64]: "f64",
};

/**
 * @param {number} type_info
 * @returns string
 */
export function printTypeInfo(type_info) {
	let hint = type_info & TYPE_HINT_MASK;
	let dim = type_info & TYPE_DIM_MASK;
	let prim = type_info & TYPE_PRIM_MASK;

	if (hint == TYPE_HINT_NONE) {
		return `${tylookup_prim[prim] ?? prim}:${tylookup_dim[dim] ?? (dim >> 4)}`;
	}
	return `${tylookup_prim[prim] ?? prim}:${tylookup_dim[dim] ?? (dim >> 4)}:${tylookup_hint[hint] ?? (hint >> 8)}`;
}

export class BlobIO {
	/**
	 * Constructor.
	 * @param {Blob} blob
	 */
	constructor(blob) {
		this.blob = blob;
		this.size = blob.size;
	}
	/**
	 * @param {number} start
	 * @param {number} size
	 * @returns ArrayBuffer
	 */
	async read(start, size) {
		return await this.blob.slice(start, start + size).arrayBuffer();
	}
}

export class UrlIO {
	/**
	 * Constructor.
	 * @param {string} url
	 * @param {{ [key: string]: string; }} headers
	 */
	constructor(url, headers = {}) {
		this.url = url;
		this.headers = headers;
		this.size = null;
	}
	async head() {

	}
	/**
	 * @param {number} start
	 * @param {number} size
	 * @returns ArrayBuffer
	 */
	async read(start, size) {
		if (size == 0) {
			return new ArrayBuffer(0);
		}
		let response = await fetch(this.url, {
			method: 'GET',
			headers: {
				'Range': `bytes=${start}-${start + size - 1}`,
				...this.headers
			},
		});
		let buffer = await response.arrayBuffer();
		if (buffer.byteLength != size) {
			buffer = buffer.slice(0, size);
		}
		return buffer;
	}
}

function parseDatasetStaticHeader(buffer, dataset_size) {
	if (buffer.byteLength < 0x18) {
		throw Error("Invalid Dataset: Insufficient bytes for static header");
	}

	// Parse the static header
	let view = new DataView(buffer);
	let check = view.getUint32(0, true);
	if (check != 0x7fcea59b) {
		throw Error("Invalid Dataset: Incorrect check value");
	}

	// let head_checksum = view.getUint32(4, true);
	let id = decodeId(view.getUint32(8, true));
	let header_size = view.getUint16(12, true);
	let descs_len = view.getUint16(14, true);
	let lookups_len = view.getUint16(16, true);
	let string_len = view.getUint16(18, true);

	// Header must fit inside the dataset
	if (header_size > dataset_size) {
		throw Error("Invalid Dataset: Header size too large");
	}

	// Alignment checks
	if ((header_size % 8) != 0 || (string_len % 8) != 0) {
		throw Error("Invalid Dataset: Header misaligned size")
	}

	// Header must contain all the necessary structures
	if (header_size < 0x18 + descs_len * 0x30 + lookups_len * 0x8 + string_len) {
		throw Error("Invalid Dataset: Header size too small");
	}

	return Object.freeze({
		id,
		header_size,
		descs_len,
		lookups_len,
		string_len,
	});
}
function parseDatasetHeader(file_offset, buffer, s) {
	if (buffer.byteLength < s.header_size) {
		throw Error("Invalid Dataset: Insufficient bytes for header");
	}
	let view = new DataView(buffer, 0, s.header_size);

	// Parse the strings
	let strings = {};
	{
		let offset = 0x18 + s.descs_len * 0x30;
		let sbase = offset + s.lookups_len * 0x8;
		for (let i = 0; i < s.lookups_len; i += 1) {
			let hash = view.getUint32(offset + 0, true);
			let start = view.getUint16(offset + 4, true);
			let slen = view.getUint16(offset + 6, true);
			offset += 0x8;
			if (start + slen > string_len) {
				throw Error("Invalid Dataset String: Out of bounds");
			}
			strings[hash] = utf8Decoder.decode(new Uint8Array(buffer, sbase + start, slen));
		}
	}

	let nm = name => name == 0 ? null : strings[name] ?? `0x${name.toString(16)}`;

	// Parse the datatables
	let datatables = [];
	for (let i = 0; i < s.descs_len; i += 1) {
		let base = 0x18 + i * 0x30;
		let key_name = nm(view.getUint32(base + 0x00, true));
		if (key_name == null) {
			throw Error("Invalid Dataset Datatable: Null key_name");
		}
		let type_info = view.getUint16(base + 0x04, true);
		let compress_info = view.getUint16(base + 0x06, true);
		let mem_start = view.getUint32(base + 0x08, true);
		let mem_end = view.getUint32(base + 0x0C, true);
		let data_size = view.getUint32(base + 0x10, true);
		let data_shape0 = view.getUint32(base + 0x14, true);
		let data_shape1 = view.getUint32(base + 0x18, true);
		let data_shape = decodeShape(type_info, data_shape0, data_shape1);
		let index_name = nm(view.getUint32(base + 0x1C, true));
		let related_name = nm(view.getUint32(base + 0x20, true));
		let type_name = nm(view.getUint32(base + 0x24, true));
		// let checksum = nm(view2.getUint32(base + 0x28, true));
		let data_offset = file_offset.offset + s.head_size + mem_start * 8;
		datatables.push(Object.freeze({
			key_name,
			type_info,
			compress_info,
			mem_start,
			mem_end,
			data_size,
			data_shape,
			index_name,
			related_name,
			type_name,
			file_offset: Object.freeze({
				offset: data_offset,
				size: data_size,
			}),
		}));
	}

	return Object.freeze({
		file_offset: file_offset,
		id: s.id,
		header_size: s.header_size,
		tables: datatables,
	});
}

export class UdfReader {
	/**
	 * Constructor.
	 * @param {UrlIO|BlobIO} io
	 */
	constructor(io) {
		this.io = io;
	}

	/**
	 * Reads and parses the file header.
	 * @returns Promise<{ id: string, next: number, root: { offset: number, size: number } }>
	 */
	async readFileHeader() {
		let buffer = await this.io.read(0, 0x40);
		if (buffer.byteLength < 0x40) {
			throw Error("Invalid UDF: Insufficient bytes for header");
		}

		let view = new DataView(buffer);
		let magic = view.getUint32(0, true);
		if (magic != 0x30464455) { // 'UDF0'
			throw Error("Invalid UDF: Unknown magic number");
		}

		let id = decodeId(view.getUint32(4, true));
		let next = Number(view.getBigUint64(8, true));
		let root_offset = Number(view.getBigUint64(16, true));
		let root_size = Number(view.getBigUint64(24, true));

		let res0 = Number(view.getBigUint64(30, true));
		let res1 = Number(view.getBigUint64(38, true));
		let res2 = Number(view.getBigUint64(46, true));
		let res3 = Number(view.getBigUint64(54, true));
		if (res0 != 0 || res1 != 0 || res2 != 0 || res3 != 0) {
			throw Error("Invalid UDF: Reserved fields must be zero");
		}

		return Object.freeze({
			magic: 'UDF0',
			id,
			next,
			root: Object.freeze({
				offset: root_offset,
				size: root_size,
			}),
		})
	}

	/**
	 * Reads and parses an UDF dataset header.
	 * @param {{ offset: number; size: number; }} file_offset
	 */
	async readDatasetHeader(file_offset) {
		// Size of the static header
		if (file_offset.size < 0x18 || file_offset.size % 16 != 0 || file_offset.offset == 0 || file_offset.offset % 16 != 0) {
			throw Error("Invalid Dataset: Invalid file offset");
		}

		// Read the static header
		let buffer = await this.io.read(file_offset.offset, 0x18);
		if (buffer.byteLength < 0x18) {
			throw Error("Invalid Dataset: Insufficient bytes for static header");
		}

		// Parse the static header
		let view = new DataView(buffer);
		let check = view.getUint32(0, true);
		if (check != 0x7fcea59b) {
			throw Error("Invalid Dataset: Incorrect check value");
		}

		// let head_checksum = view.getUint32(4, true);
		let id = decodeId(view.getUint32(8, true));
		let head_size = view.getUint16(12, true);
		let descs_len = view.getUint16(14, true);
		let lookups_len = view.getUint16(16, true);
		let string_len = view.getUint16(18, true);

		if (file_offset.size < head_size) {
			throw Error("Invalid Dataset: Header too large");
		}

		if (head_size < 0x18 + descs_len * 0x30 + lookups_len * 0x8 + string_len) {
			throw Error("Invalid Dataset: Header too small");
		}

		if (string_len % 8 != 0) {
			throw Error("")
		}

		// Read the full header
		if (buffer.byteLength < head_size) {
			buffer = await this.io.read(file_offset.offset, head_size);
			if (buffer.byteLength < head_size) {
				throw Error("Invalid Dataset: Insufficient bytes for header");
			}
			view = new DataView(buffer);
		}

		// Parse the strings
		let strings = {};
		{
			let offset = 0x18 + descs_len * 0x30;
			let sbase = offset + lookups_len * 0x8;
			for (let i = 0; i < lookups_len; i += 1) {
				let hash = view.getUint32(offset + 0, true);
				let start = view.getUint16(offset + 4, true);
				let slen = view.getUint16(offset + 6, true);
				offset += 0x8;
				if (start + slen > string_len) {
					throw Error("Invalid Dataset String: Out of bounds");
				}
				strings[hash] = utf8Decoder.decode(new Uint8Array(buffer, sbase + start, slen));
			}
		}

		let nm = name => name == 0 ? null : strings[name] ?? `0x${name.toString(16)}`;

		// Parse the datatables
		let datatables = [];
		for (let i = 0; i < descs_len; i += 1) {
			let base = 0x18 + i * 0x30;
			let key_name = nm(view.getUint32(base + 0x00, true));
			if (key_name == null) {
				throw Error("Invalid Dataset Datatable: Null key_name");
			}
			let type_info = view.getUint16(base + 0x04, true);
			let compress_info = view.getUint16(base + 0x06, true);
			let mem_start = view.getUint32(base + 0x08, true);
			let mem_end = view.getUint32(base + 0x0C, true);
			let data_size = view.getUint32(base + 0x10, true);
			let data_shape0 = view.getUint32(base + 0x14, true);
			let data_shape1 = view.getUint32(base + 0x18, true);
			let data_shape = decodeShape(type_info, data_shape0, data_shape1);
			let index_name = nm(view.getUint32(base + 0x1C, true));
			let related_name = nm(view.getUint32(base + 0x20, true));
			let type_name = nm(view.getUint32(base + 0x24, true));
			// let checksum = nm(view2.getUint32(base + 0x28, true));
			let data_offset = file_offset.offset + head_size + mem_start * 8;
			datatables.push(Object.freeze({
				key_name,
				type_info,
				compress_info,
				mem_start,
				mem_end,
				data_size,
				data_shape,
				index_name,
				related_name,
				type_name,
				file_offset: Object.freeze({
					offset: data_offset,
					size: data_size,
				}),
			}));
		}

		return Object.freeze({
			file_offset: file_offset,
			id,
			header_size: head_size,
			tables: datatables,
		});
	}

	async readBuffer(file_offset) {
		return await this.io.read(file_offset.offset, file_offset.size);
	}

	async readData(desc) {
		let buffer = await this.readBuffer(desc.file_offset);
		if (desc.compress_info != 0) {
			throw Error("Decompression not implemented");
		}

		let hint = desc.type_info & TYPE_HINT_MASK;
		let prim = desc.type_info & TYPE_PRIM_MASK;

		if (hint == TYPE_HINT_TEXT) {
			return decodeHintText(desc, buffer);
		}

		if (hint == TYPE_HINT_JSON) {
			return JSON.parse((new TextDecoder('utf-8')).decode(buffer));
		}

		if (prim == TYPE_PRIM_U8) {
			return new Uint8Array(buffer);
		}
		else if (prim == TYPE_PRIM_I8) {
			return new Int8Array(buffer);
		}
		else if (prim == TYPE_PRIM_U16) {
			return new Uint16Array(buffer);
		}
		else if (prim == TYPE_PRIM_I16) {
			return new Int16Array(buffer);
		}
		else if (prim == TYPE_PRIM_U32) {
			return new Uint32Array(buffer);
		}
		else if (prim == TYPE_PRIM_I32) {
			return new Int32Array(buffer);
		}
		else if (prim == TYPE_PRIM_U64) {
			return new BigUint64Array(buffer);
		}
		else if (prim == TYPE_PRIM_I64) {
			return new BigInt64Array(buffer);
		}
		else if (prim == TYPE_PRIM_F32) {
			return new Float32Array(buffer);
		}
		else if (prim == TYPE_PRIM_F64) {
			return new Float64Array(buffer);
		}

		return buffer;
	}
}

function decodeHintText(desc, buffer) {
	let dim = desc.type_info & TYPE_DIM_MASK;
	let prim = desc.type_info & TYPE_PRIM_MASK;

	let array, td;
	if (prim == TYPE_PRIM_U8 || prim == TYPE_PRIM_I8) {
		array = new Uint8Array(buffer);
		td = utf8Decoder;
	}
	else if (prim == TYPE_PRIM_U16) {
		array = new Uint16Array(buffer);
		td = new TextDecoder('utf-16');
	}
	else if (prim == TYPE_PRIM_U32) {
		array = new Uint32Array(buffer);
		td = { decode(array) { return String.fromCodePoint(...array); } };
	}
	else {
		throw TypeError("");
	}

	let shape = desc.data_shape;

	if (dim == TYPE_DIM_SCALAR) {
		return td.decode(array);
	}

	if (dim == TYPE_DIM_1D) {
		// Must have 1 ghost dimension indicating the element width of each row
		// Strings are extracted by nul terminator
		if (shape.length != 2) {
			throw TypeError();
		}
		let strings = [];
		let width = shape[1];
		for (let i = 0; i < shape[0]; i += 1) {
			let start = i * width;
			let slen = width;
			while (slen > 0 && array[start + slen - 1] == 0) {
				slen -= 1;
			}
			let s = td.decode(array.subarray(start, start + slen));
			strings.push(s);
		}
		return strings;
	}

	if (dim == TYPE_DIM_2D) {
		throw Error("Not implemented");
	}

	throw TypeError();
}
