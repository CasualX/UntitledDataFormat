
// This little example generates an invalid UDF file
// Its purpose is to build test cases for validation
fn main() {
	let mut file = udf::FileIO::create("invalid.udf", [0; 4]).unwrap();

	file.write_header().unwrap();
}


/*
Ideas for invalid datasets:

* Misaligned file offset offset
* Misaligned file offset size
* Dataset with wrong check value
* Dataset with incorrect checksums
* Dataset with no names

*/
