use super::*;

const OP_DELTA1: u8 = 0b00_000000; // delta of -32..32 (1 byte)
const OP_DELTA2: u8 = 0b01_000000; // delta of -8192..8192 (2 byte)
const OP_INDEX: u8  = 0b10_000000; // 6-bit index (1 byte)
const OP_DELTA3: u8 = 0b1100_0000; // delta of -524288..524288 (3 byte)
const OP_DELTA4: u8 = 0b1101_0000; // delta of -134217728..134217728 (4 byte)
const OP_REPEAT: u8 = 0b1110_0000; // repeat last value (up to 16 times) (1 byte)
const OP_VALUES: u8 = 0b1111_0000; // copy uncompressed values (up to 16 vals) (1 + n byte)
const OP_DELTA1_BITS: usize = 6;
const OP_DELTA2_BITS: usize = 6 + 8;
const OP_DELTA3_BITS: usize = 4 + 8 + 8;
const OP_DELTA4_BITS: usize = 4 + 8 + 8 + 8;
const OP_DELTA1_VAL: i32 = (1 << OP_DELTA1_BITS) / 2;
const OP_DELTA2_VAL: i32 = (1 << OP_DELTA2_BITS) / 2;
const OP_DELTA3_VAL: i32 = (1 << OP_DELTA3_BITS) / 2;
const OP_DELTA4_VAL: i32 = (1 << OP_DELTA4_BITS) / 2;
const OP_REPEAT_MAX: usize = 16;
const OP_VALUES_MAX: usize = 16;

/// Simple lossy compression scheme for 32-bit floats.
#[derive(Copy, Clone, Debug)]
pub struct SimpleF32 {
	pub unit: f32,
}

impl SimpleF32 {
	#[inline]
	pub fn compress(&self, storage: &mut Vec<u8>, data: &[f32]) {
		compress(storage, data, self.unit)
	}
	#[inline]
	pub fn decompress(storage: &mut [f32], stream: &[u8]) -> bool {
		decompress(storage, stream)
	}
}

fn compress(buf: &mut Vec<u8>, data: &[f32], unit: f32) {
	let inv_unit = 1.0 / unit;
	let ule = unit.to_le_bytes();
	buf.push(ule[0]);
	buf.push(ule[1]);
	buf.push(ule[2]);
	buf.push(ule[3]);

	// Compression state
	let mut lastv = 0u32; // Last value for RLE and delta
	let mut run = 0usize; // RLE counter
	let mut unc = 0usize; // uncompressed values counter
	let mut lookup = [0u32; 64]; // lookup table for index

	for i in 0..data.len() {
		let v = (data[i] * inv_unit).round() as i32 as u32;

		// Check for repeated value
		if v == lastv {
			run += 1;
			// Check if max run of repeat
			if run == OP_REPEAT_MAX {
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
			let index = hash32(v) as usize % lookup.len();
			if lookup[index] == v {
				buf.push(OP_INDEX | index as u8);
				unc = 0;
			}
			else {
				lookup[index] = v;

				// Check for delta to last value
				let mut dv = v.wrapping_sub(lastv) as i32;
				if dv > 0 { dv -= 1; } // case of delta = 0 already handled by RLE

				// Small delta
				if dv >= -OP_DELTA1_VAL && dv < OP_DELTA1_VAL {
					buf.push(OP_DELTA1 | (dv & 0b00_111111) as u8);
					unc = 0;
				}
				// Medium delta
				else if dv >= -OP_DELTA2_VAL && dv < OP_DELTA2_VAL {
					buf.push(OP_DELTA2 | ((dv >> 8) & 0b00_111111) as u8);
					buf.push((dv & 0xff) as u8);
					unc = 0;
				}
				// Large delta
				else if dv >= -OP_DELTA3_VAL && dv < OP_DELTA3_VAL {
					buf.push(OP_DELTA3 | ((dv >> 8 + 8) & 0b0000_1111) as u8);
					buf.push((dv >> 8 & 0xff) as u8);
					buf.push((dv & 0xff) as u8);
				}
				// Largest delta
				else if dv >= -OP_DELTA4_VAL && dv < OP_DELTA4_VAL {
					buf.push(OP_DELTA4 | ((dv >> 8 + 8 + 8) & 0b0000_1111) as u8);
					buf.push((dv >> 8 + 8 & 0xff) as u8);
					buf.push((dv >> 8 & 0xff) as u8);
					buf.push((dv & 0xff) as u8);
				}
				else {

					// Start uncompressed values
					if unc == 0 || unc == OP_VALUES_MAX {
						buf.push(OP_VALUES);
						unc = 0;
					}
					// Increment run of uncompresed values
					else {
						let pos = buf.len() - (1 + unc * 4);
						buf[pos] += 1;
					}

					// Write uncompressed value
					let vle = v.to_le_bytes();
					buf.push(vle[0]);
					buf.push(vle[1]);
					buf.push(vle[2]);
					buf.push(vle[3]);
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

fn decompress(storage: &mut [f32], stream: &[u8]) -> bool {
	if stream.len() < 4 {
		return false;
	}
	let unit = f32::from_le_bytes([stream[0], stream[1], stream[2], stream[3]]);

	let mut lastv = 0u32;
	let mut lookup = [0u32; 64];

	let mut i = 4;
	let mut k = 0;
	while i < stream.len() {
		let byte = stream[i];
		i += 1;

		// Decode the compression opcode into a helper enum
		enum OpCode { Delta1, Delta2, Delta3, Delta4, Index, Repeat, Values }
		let op = if byte & 0b11_000000 == OP_DELTA1 { OpCode::Delta1 }
		    else if byte & 0b11_000000 == OP_DELTA2 { OpCode::Delta2 }
		    else if byte & 0b11_000000 == OP_INDEX  { OpCode::Index  }
		    else if byte & 0b1111_0000 == OP_DELTA3 { OpCode::Delta3 }
		    else if byte & 0b1111_0000 == OP_DELTA4 { OpCode::Delta4 }
		    else if byte & 0b1111_0000 == OP_REPEAT { OpCode::Repeat }
		    else if byte & 0b1111_0000 == OP_VALUES { OpCode::Values }
		    else { unreachable!() };

		match op {
			OpCode::Delta1 | OpCode::Delta2 | OpCode::Delta3 | OpCode::Delta4 => {
				let mut dv = match op {
					OpCode::Delta1 => {
						sign_extend32((byte & 0b00_111111) as u32, OP_DELTA1_BITS)
					},
					OpCode::Delta2 => {
						let &byte2 = some!(stream.get(i + 0));
						i += 1;
						sign_extend32(((byte & 0b00_111111) as u32) << 8 | (byte2 as u32), OP_DELTA2_BITS)
					},
					OpCode::Delta3 => {
						let &byte3 = some!(stream.get(i + 1));
						let &byte2 = some!(stream.get(i + 0));
						i += 2;
						sign_extend32(((byte & 0b0000_1111) as u32) << 8 + 8 | (byte2 as u32) << 8 | (byte3 as u32), OP_DELTA3_BITS)
					},
					OpCode::Delta4 => {
						let &byte4 = some!(stream.get(i + 2));
						let &byte3 = some!(stream.get(i + 1));
						let &byte2 = some!(stream.get(i + 0));
						i += 3;
						sign_extend32(((byte & 0b0000_1111) as u32) << 8 + 8 + 8 | (byte2 as u32) << 8 + 8 | (byte3 as u32) << 8 | (byte4 as u32), OP_DELTA4_BITS)
					},
					_ => unreachable!()
				};
				if dv >= 0 {
					dv += 1;
				}
				let v = lastv.wrapping_add(dv as u32);

				if k >= storage.len() {
					return false;
				}
				storage[k] = v as i32 as f32 * unit;
				k += 1;

				lastv = v;

				let index = hash32(lastv) as usize % lookup.len();
				lookup[index] = lastv;
			},
			OpCode::Index => {
				let index = (byte & 0b00_111111) as usize;
				let v = lookup[index];

				if k >= storage.len() {
					return false;
				}
				storage[k] = v as i32 as f32 * unit;
				k += 1;

				lastv = v;
			},
			OpCode::Repeat => {
				let count = (byte & 0b0000_1111) as usize + 1;

				if count > storage.len() - k {
					return false;
				}

				for _ in 0..count {
					storage[k] = lastv as i32 as f32 * unit;
					k += 1;
				}

				// Non-max repeat increment lastv by 1
				// Helps compressing runs of incrementing integers
				if count != OP_REPEAT_MAX {
					lastv = lastv.wrapping_add(1);
				}
			},
			OpCode::Values => {
				let count = (byte & 0b0000_1111) as usize + 1;

				for _ in 0..count {
					if 4 > stream.len() - i {
						return false;
					}
					let v = u32::from_le_bytes([stream[i + 0], stream[i + 1], stream[i + 2], stream[i + 3]]);
					i += 4;

					if k >= storage.len() {
						return false;
					}
					storage[k] = v as i32 as f32 * unit;
					k += 1;

					lastv = v;

					let index = hash32(lastv) as usize % lookup.len();
					lookup[index] = lastv;
				}
			},
		}
	}

	// Decompression is successful only if the whole storage was filled
	return k == storage.len();
}
