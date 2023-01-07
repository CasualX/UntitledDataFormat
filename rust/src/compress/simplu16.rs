use super::*;

const OP_XDELTA: u8 = 0b00_000000; // (0) delta of -32..32 (1 byte)
const OP_YDELTA: u8 = 0b01_000000; // (1) delta of -8192..8192 (2 byte)
const OP_INDEX: u8  = 0b10_000000; // (2) 6-bit index (1 byte)
const OP_REPEAT: u8 = 0b110_00000; // (3) repeat last value (up to 32 times) (1 byte)
const OP_VALUES: u8 = 0b111_00000; // (4) copy uncompressed values (up to 32 vals) (1 + n byte)

/// Simple lossless compression scheme for 16-bit integers.
#[derive(Copy, Clone, Debug)]
pub struct SimpleU16;

impl SimpleU16 {
	#[inline]
	pub fn compress(storage: &mut Vec<u8>, data: &[u16]) {
		compress(storage, data)
	}
	#[inline]
	pub fn decompress(storage: &mut [u16], stream: &[u8]) -> bool {
		decompress(storage, stream)
	}
}

fn compress(buf: &mut Vec<u8>, data: &[u16]) {
	// Compression state
	let mut lastv = 0u16; // Last value for RLE and delta
	let mut run = 0usize; // RLE counter
	let mut unc = 0usize; // uncompressed values counter
	let mut lookup = [0u16; 64]; // lookup table for index

	for i in 0..data.len() {
		let v = data[i];

		// Check for repeated value
		if v == lastv {
			run += 1;
			// Check if max run of repeat
			if run == 32 {
				buf.push(OP_REPEAT | (run - 1) as u8);
				run = 0;
				unc = 0;
			}
		}
		else {
			// If no more repeated values
			if run > 0 {
				buf.push(OP_REPEAT | (run - 1) as u8);
				run = 0;
				unc = 0;

				// Non-max repeat increment lastv by 1
				// Helps compressing runs of incrementing integers
				lastv = lastv.wrapping_add(1);
				if v == lastv {
					run = 1;
					continue;
				}
			}

			// Check if value in lookup
			let index = hash16(v) as usize % lookup.len();
			if lookup[index] == v {
				buf.push(OP_INDEX | index as u8);
				unc = 0;
			}
			else {
				lookup[index] = v;

				// Check for delta to last value
				let mut dv = v.wrapping_sub(lastv) as i32;
				if dv > 0 { dv -= 1; } // case of delta = 0 already handled by RLE

				// Small xdelta
				if dv >= -32 && dv < 32 {
					buf.push(OP_XDELTA | (dv & 0b00_111111) as u8);
					unc = 0;
				}
				// Medium ydelta
				else if dv >= -8192 && dv < 8192 {
					buf.push(OP_YDELTA | ((dv >> 8) & 0b00_111111) as u8);
					buf.push((dv & 0xff) as u8);
					unc = 0;
				}
				else {

					// Start uncompressed values
					if unc == 0 || unc == 32 {
						buf.push(OP_VALUES);
						unc = 0;
					}
					// Increment run of uncompresed values
					else {
						let pos = buf.len() - (1 + unc * 2);
						buf[pos] += 1;
					}

					// Write uncompressed value
					let vle = v.to_le_bytes();
					buf.push(vle[0]);
					buf.push(vle[1]);
				}
			}
		}

		lastv = v;
	}

	// Make sure the last run is added to the storage
	if run > 0 {
		buf.push(OP_REPEAT | (run - 1) as u8);
	}
}

fn decompress(storage: &mut [u16], stream: &[u8]) -> bool {
	let mut lastv = 0u16;
	let mut lookup = [0u16; 64];

	let mut i = 0;
	let mut k = 0;
	while i < stream.len() {
		let byte = stream[i];
		i += 1;

		if byte & 0b11_000000 == OP_XDELTA {
			let mut dv = sign_extend32(byte as u32, 6) as i32;
			if dv >= 0 {
				dv += 1;
			}
			let v = lastv.wrapping_add(dv as u16);

			if k >= storage.len() {
				return false;
			}
			storage[k] = v;
			k += 1;

			lastv = v;

			let index = hash16(lastv) as usize % lookup.len();
			lookup[index] = lastv;
		}

		else if byte & 0b11_000000 == OP_YDELTA {
			if i >= stream.len() {
				return false;
			}
			let mut dv = sign_extend32((byte as u32) << 8 | stream[i] as u32, 14);
			i += 1;

			if dv >= 0 {
				dv += 1;
			}
			let v = lastv.wrapping_add(dv as u16);

			if k >= storage.len() {
				return false;
			}
			storage[k] = v;
			k += 1;

			lastv = v;

			let index = hash16(lastv) as usize % lookup.len();
			lookup[index] = lastv;
		}

		else if byte & 0b11_000000 == OP_INDEX {
			let index = (byte & 0b00_111111) as usize;
			let v = lookup[index];

			if k >= storage.len() {
				return false;
			}
			storage[k] = v;
			k += 1;

			lastv = v;
		}

		else if byte & 0b111_00000 == OP_REPEAT {
			let count = (byte & 0b000_11111) as usize + 1;

			if count > storage.len() - k {
				return false;
			}

			for _ in 0..count {
				storage[k] = lastv;
				k += 1;
			}

			// Non-max repeat increment lastv by 1
			// Helps compressing runs of incrementing integers
			if count != 32 {
				lastv = lastv.wrapping_add(1);
			}
		}

		else {
			let count = (byte & 0b000_11111) as usize + 1;

			for _ in 0..count {
				if 2 > stream.len() - i {
					return false;
				}
				let v = u16::from_le_bytes([stream[i + 0], stream[i + 1]]);
				i += 2;

				if k >= storage.len() {
					return false;
				}
				storage[k] = v;
				k += 1;

				lastv = v;

				let index = hash16(lastv) as usize % lookup.len();
				lookup[index] = lastv;
			}
		}
	}

	// Decompression is successful only if the whole storage was filled
	return k == storage.len();
}

#[test]
fn regressions() {
	// bug: last RLE was not written for RLE into RLE of 1 byte.
	#[cfg(test)]
	static REGRESSION1: [u16; 2] = [0, 1];
	// bug: not writing uncompressed values into lookup
	#[cfg(test)]
	static REGRESSION2: [u16; 4] = [16209, 59, 3994, 59];

	let cases = [
		&REGRESSION1[..],
		&REGRESSION2[..],
	];

	for data in cases {
		let mut stream = Vec::new();
		compress(&mut stream, data);
		let stream = stream.as_slice();
		println!("{:x?}", stream);
		let mut storage = vec![0u16; data.len()];
		decompress(&mut storage, stream);
		assert_eq!(&storage, data);
	}
}
