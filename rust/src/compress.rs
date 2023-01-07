
macro_rules! some {
	($expr:expr) => {
		match $expr {
			Some(val) => val,
			None => return false,
		}
	};
}

mod simplu32;
mod simplu16;
mod simpf32;

use dataview::Pod;

pub use self::simplu32::SimpleU32;
pub use self::simpf32::SimpleF32;
pub use self::simplu16::SimpleU16;

#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct Stats {
	pub xdelta: u32,
	pub ydelta: u32,
	pub index: u32,
	pub repeat: u32,
	pub values: u32,
	pub ratio: f32,
}

pub struct EncodeBuffer<'a> {
	pub storage: &'a mut Vec<u64>,
	pub byte_offset: usize,
}
impl<'a> EncodeBuffer<'a> {
	pub fn push(&mut self, byte: u8) {
		if self.byte_offset % 8 == 0 || self.storage.len() == 0 {
			self.storage.push(byte as u64);
			self.byte_offset += 1;
		}
		else {
			*self.storage.last_mut().unwrap() |= (byte as u64) << self.byte_offset % 8 * 8;
			self.byte_offset += 1;
		}
	}
	pub fn inc(&mut self, last: usize) {
		let len = (self.storage.len() - 1) * 8 + self.byte_offset;
		let storage = dataview::bytes_mut(self.storage.as_mut_slice());
		storage[len - last] += 1;
		// unimplemented!()
	}
}

fn hash16(a: u16) -> u16 {
	hash32(a as u32) as u16
}
// https://burtleburtle.net/bob/hash/integer.html
pub fn hash32(mut a: u32) -> u32 {
	a = (a ^ 61) ^ (a >> 16);
	a = a.wrapping_add(a << 3);
	a = a ^ (a >> 4);
	a = a.wrapping_mul(0x27d4eb2d);
	a = a ^ (a >> 15);
	return a;
}
// Java's SplitMix64 PRNG
fn hash64(mut z: u64) -> u64 {
	z = z.wrapping_add(0x9e3779b97f4a7c15);
	z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
	z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
	return z ^ (z >> 31);
}
fn castu16(slice: &[i16]) -> &[u16] {
	unsafe { std::mem::transmute(slice) }
}
fn castu32(slice: &[i32]) -> &[u32] {
	unsafe { std::mem::transmute(slice) }
}

const fn sign_extend32(int: u32, bits: usize) -> i32 {
	let mask = (1u32 << bits) - 1;
	(if int & (1 << (bits - 1)) == 0 { int & mask }
	else { int | !mask }) as i32
}
