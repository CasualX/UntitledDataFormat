
/// Compiletime string constant hash.
///
/// Implemented using the [DJB2 hash function](http://www.cse.yorku.ca/~oz/hash.html#djb2) xor variation.
#[inline(always)]
pub const fn hash(s: &str) -> u32 {
	let s = s.as_bytes();
	let mut result = 3581u32;
	let mut i = 0usize;
	while i < s.len() {
		result = result.wrapping_mul(33) ^ s[i] as u32;
		i += 1;
	}
	return result;
}

/// Compiletime string constant hash.
///
/// Helper macro guarantees compiletime evaluation of the string constant hash.
///
/// ```
/// const STRING: &str = "Hello World";
/// assert_eq!(udf::hash!(STRING), 0x6E4A573D);
/// ```
#[macro_export]
macro_rules! hash {
	($s:expr) => {{ const _DJB2_HASH: u32 = $crate::hash($s); _DJB2_HASH }};
}
