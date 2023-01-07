# Untitled Data Format Specification

## Introduction

All values are stored in little endian format.

Fields marked optional are still present just filled with the value `0` indicating the value is absent. This means `0` is not a valid value for these fields.

Reserved fields marked 'must be zero' must be checked by a conforming UDF parser and an error must be raised if non-zero values are found.

Types use Rust's primitive integer types and array syntax.

## File header

The UDF file starts with a fixed size header of `0x40` bytes:

| Offset | Size   | Name        | Type         | Description
|--------|--------|-------------|--------------|------------
| `0x0`  | `0x4`  | magic       | `[u8; 4]`    | File signature, must be ASCII "UDF0": `0x55 0x44 0x46 0x30`. Later revisions may increment the last digit but UDF files will always start with ASCII "UDF".
| `0x4`  | `0x4`  | id          | `[u8; 4]`    | Identifier. Helpful for identifying the intended structure its datasets.
| `0x8`  | `0x8`  | next        | `u64`        | Reserved for future use.
| `0x10` | `0x10` | root        | `FileOffset` | Optional. Location of the root dataset.
| `0x20` | `0x20` | reserved    | `[u64; 4]`   | Reserved for future use, must be zero.

The datasets are stored in no particular order or location after the file header. Datasets are always referred to by their absolute file offsets.

### Identifier

The id fields are 4 bytes and must contain only printable ASCII characters. Identifiers shorter than 4 characters are terminated by padding the rest with nul bytes.

### File offset

File offsets are used to locate datasets in the UDF file.

When both offset and size are zero the file offset is `null` and does not point to any dataset. A non-zero size is not valid when offset is zero.

| Offset | Size   | Name   | Type  | Description
|--------|--------|--------|-------|------------
| `0x0`  | `0x8`  | offset | `u64` | Byte offset from the start of the file. Must be aligned to 16.
| `0x8`  | `0x8`  | size   | `u64` | Size in bytes. Must be aligned to 16.

## Dataset

A dataset is created out of two components: a header and storage for the datatable data contents. The total size of the dataset is defined by the `FileOffset` and must fully contain the dataset (ie. all offsets must be checked to be within the bounds of the file offset).

The dataset header has a static part of `0x18` bytes:

| Offset | Size   | Name         | Type        | Description
|--------|--------|--------------|-------------|------------
| `0x0`  | `0x4`  | check        | `u32`       | Special check value for increasing confidence that this is really a dataset and not some arbitrary bytes. Must be `0x7fcea59b`.
| `0x4`  | `0x4`  | checksum     | `u32`       | Optional header checksum.
| `0x8`  | `0x4`  | id           | `[u8; 4]`   | Identifier. Helpful for identifying the intended structure of this dataset.
| `0xC`  | `0x2`  | header_size  | `u16`       | Size of the header in bytes, must be a multiple of 8.
| `0xE`  | `0x2`  | descs_len    | `u16`       | Number of datatable descriptors following the static header.
| `0x10` | `0x2`  | lookup_len   | `u16`       | Number of string lookup entries following the datatable descriptors.
| `0x12` | `0x2`  | string_len   | `u16`       | Byte length of the string following the lookup entries, must be a multiple of 8.
| `0x14` | `0x4`  | reserved     | `[u16; 2]`  | Reserved for future use.

The dataset static header is immediately followed by a number of datatable descriptors. It has a size of `0x30` bytes per descriptor:

| Offset | Size   | Name          | Type        | Description
|--------|--------|---------------|-------------|------------
| `0x0`  | `0x4`  | key_name      | `u32`       | The key name of this entry.
| `0x4`  | `0x2`  | type_info     | `u16`       | Type information about this datatable's contents.
| `0x6`  | `0x2`  | compress_info | `u16`       | Optional. Specifies the compression scheme used.
| `0x8`  | `0x4`  | mem_start     | `u32`       | See below.
| `0xC`  | `0x4`  | mem_end       | `u32`       | See below.
| `0x10` | `0x4`  | data_size     | `u32`       | Size of the data (after compression) in bytes.
| `0x14` | `0x8`  | data_shape    | `[u32; 2]`  | Shape of the data.
| `0x1C` | `0x4`  | index_name    | `u32`       | Optional. When an index type hint is used this specifies which datatable the indices go into, otherwise null.
| `0x20` | `0x4`  | related_name  | `u32`       | Optional. Struct of Arrays (SoA) indicates related datatable.
| `0x24` | `0x4`  | type_name     | `u32`       | Optional. Additional type info encoded as a free-form string.
| `0x28` | `0x4`  | checksum      | `u32`       | Optional. Checksum of the data referenced by `mem_start` and `mem_end`.
| `0x2C` | `0x4`  | reserved      | `[u32; 1]`  | Reserved for future use.

The `mem_start` and `mem_end` fields are not in bytes, but rather in 'blocks' of 8 bytes. Multiply these values by 8 to get the byte offset. `mem_end` must be larger or equal to `mem_start`. These offsets start _after_ the header (NOT! the start of the dataset file offset, this simplifies building datasets). Eg. `mem_start` of `0` starts their data on dataset file offset + `header_size`. Note that the header size is a multiple of 8 so all offsets end up aligned to 8 bytes.

This implies that datasets have a limit of `32 GiB` and each datatable has a limit of `4 GiB`. If this is insufficient consider manually splitting the data over multiple datasets.

Strings such as `key_name`, `index_name`, `related_name` and `type_name` are keys in the list of string lookup entries following the datatable descriptors:

| Offset | Size   | Name   | Type  | Description
|--------|--------|--------|-------|------------
| `0x0`  | `0x4`  | hash   | `u32` | A number associated with the string, it may be the hash of the string but it is not required. Must not be zero.
| `0x4`  | `0x2`  | offset | `u16` | Byte offset to the utf-8 encoded string. The offset starts after the string entries.
| `0x6`  | `0x2`  | len    | `u16` | Byte length of the string.

Following the string lookup entries is a single utf-8 encoded string containing all the substrings referenced by the string lookup entries concatenated. The size of this string is `string_len` stored in the static header.

### Type info

The type info contains the primitive type, the number of dimensions and a type hint.

The following diagram shows the information packed in the type info field:

```
+- type_info -------------------------------+
|      Byte[1]      |        Byte[0]        |
| 7 6 | 5 4 3 2 1 0 | 7 | 6 | 5 4 | 3 2 1 0 |
|-------------------|---|---|-----|---------|
|     |        hint | x |   | dim |    prim |
+-------------------------------------------+
```

Bit 6 of `Byte[0]` is reserved and must be zero.

Bits 6 and 7 of `Byte[1]` are reserved and must be zero.

#### Primitives

The primitive bits (and the extension bit X) are defined as follows:

| X   | Value  | Name               | Description
|-----|--------|--------------------|------------
| `0` | `0x00` | `TYPE_PRIM_CUSTOM` | Custom interpretation
|     | `0x01` |                    | _Reserved_
|     | `0x02` | `TYPE_PRIM_U8`     | Unsigned byte
|     | `0x03` | `TYPE_PRIM_I8`     | Signed byte
|     | `0x04` | `TYPE_PRIM_U16`    | Unsigned 16-bit int
|     | `0x05` | `TYPE_PRIM_I16`    | Signed 16-bit int
|     | `0x06` | `TYPE_PRIM_U32`    | Unsigned 32-bit int
|     | `0x07` | `TYPE_PRIM_I32`    | Signed 32-bit int
|     | `0x08` | `TYPE_PRIM_U64`    | Unsigned 64-bit int
|     | `0x09` | `TYPE_PRIM_I64`    | Signed 64-bit int
|     | `0x0A` | `TYPE_PRIM_F32`    | 32-bit float
|     | `0x0B` | `TYPE_PRIM_F64`    | 64-bit float
|     | `0x0C` |                    | _Reserved_
|     | `0x0D` |                    | _Reserved_
|     | `0x0E` |                    | _Reserved_
|     | `0x0F` |                    | _Reserved_
| `1` | ..     |                    | _Reserved_

#### Dimensions

There are four supported dimension values:

| Value  | Name              | Description
|--------|-------------------|------------
| `0x00` | `TYPE_DIM_SCALAR` | A single scalar value.
| `0x10` | `TYPE_DIM_1D`     | A one dimensional array.
| `0x20` | `TYPE_DIM_2D`     | A two dimensional array.
| `0x30` | `TYPE_DIM_3D`     | A three dimensional array.

#### Hints

Type hints imbue the array with additional context to help interpret its values. The datatable must remain valid without its type hint.

| Value    | Name                  | Description
|----------|-----------------------|------------
|  `0 << 8` | `TYPE_HINT_NONE`      | No type hint.
|  `1 << 8` | `TYPE_HINT_TEXT`      | Text.
|  `2 << 8` | `TYPE_HINT_JSON`      | Utf-8 encoded JSON.
|  `3 << 8` | `TYPE_HINT_DATASET`   | File offsets to other datasets.
|  `4 << 8` | `TYPE_HINT_INDEX`     | Indices into another datatable.
|  `5 << 8` | `TYPE_HINT_RANGE`     | Start, end indices into another datatable.
|  `6 << 8` | `TYPE_HINT_COORD`     | Coordinate data.
|  `7 << 8` | `TYPE_HINT_LINE`      | Line segment.
|  `8 << 8` | `TYPE_HINT_TRANSFORM` | Transformation matrix.
|  `9 << 8` | `TYPE_HINT_RGB`       | Pixel RGB colors.

Type hints above `32 << 8` can be used freely with a custom interpretation, all other values below are reserved for future use.

* `TYPE_HINT_NONE`

  There is no special interpretation, the datatable is a single, array or multi-dimensional array of primitive values.

  Ghost dimensions are not allowed.

* `TYPE_HINT_TEXT`

  The datatable contains text.

  The encoding depends on the primitive type: `TYPE_PRIM_U8`, `TYPE_PRIM_I8` are utf-8, `TYPE_PRIM_U16` is utf-16le and `TYPE_PRIM_U32` is utf-32le. No other primitive types are allowed.

  Must have a single ghost dimension.

  If the datatable is not a scalar and a string element is not as long as its longest element then it must be padded with nul characters.
  The result is a square grid of characters.

* `TYPE_HINT_JSON`

  The datatable contains a utf-8 encoded JSON document.

  The primitive type must be `TYPE_PRIM_CUSTOM` and its dimensions and shape refer to the JSON value as an array. This allows JSON values to still participate in the shape and dimension checking.

  Ghost dimensions are not allowed.

* `TYPE_HINT_DATASET`

  The datatable contains file offsets to other datasets.

  The primitive type must be `TYPE_PRIM_U64`.

  Must have a single ghost dimension with value `2` representing the `offset` and `size` fields of the file offset information.

* `TYPE_HINT_INDEX`

  The datatable contains indices into another datatable.

  The primitive type must be one of `TYPE_PRIM_U8`, `TYPE_PRIM_U16`, `TYPE_PRIM_U32`, `TYPE_PRIM_U64`.

  The index relationship is required. The target datatable must be `TYPE_DIM_1D`.

  All values in this datatable must be less than the value of the target datatable's first dimension `shape[0]`.

* `TYPE_HINT_RANGE`

  The datatable contains start, end ranges into another datatable.

  The primitive type must be one of `TYPE_PRIM_U8`, `TYPE_PRIM_U16`, `TYPE_PRIM_U32`, `TYPE_PRIM_U64`.

  The datatable must have a single ghost dimension of shape `2`. The first values is the start index and the second value is the exclusive end index.

  The index relationship is required. The target datatable must be `TYPE_DIM_1D`.

  All start values must be less than or equal to the end values. The `start..end` slice must be a valid range into the target datatable. Both values must be less than or equal to the target datatable's first dimension `shape[0]`.

* `TYPE_HINT_COORD`

  The datatable contains coordinate data.

  The primitive type must be one of `TYPE_PRIM_I8`, `TYPE_PRIM_I16`, `TYPE_PRIM_I32`, `TYPE_PRIM_I64`, `TYPE_PRIM_F32`, `TYPE_PRIM_F64`.

  The datatable must have a single ghost dimension. Its value indicates the number of dimensions of the coordinate. Eg. a value of `2` means 2d coordinate data and a value of `3` means 3d coordinate data.

* `TYPE_HINT_LINE`

  The datatable contains line segments.

  The primitive type must be one of `TYPE_PRIM_F32`, `TYPE_PRIM_F64`.

  TODO!

* `TYPE_HINT_TRANSFORM`

  The datatable contains transformation matrices.

  The primitive type must be one of `TYPE_PRIM_F32`, `TYPE_PRIM_F64`.

  The datatable must have a two ghost dimensions. The shape of the ghost dimensions is the size of the matrix.

* `TYPE_HINT_RGB`

  The datatable contains rgb color data.

  The primitive type must be one of `TYPE_PRIM_U8`, `TYPE_PRIM_F32`.

  The datatable must have a single ghost dimension. The shape of the ghost dimension must be 3 or 4.

### Type name

The type name is an optional free-form string providing additional information about the structure of the data.

### Compression

Currently not specified, future revisions will assign values to compression algorithms.

| Value | Name            | Description
|-------|-----------------|------------
| `0`   | `COMPRESS_NONE` | The data is stored without compression.

### Data shape

```
+- data_shape -------------------+
|    u32[0]    |   u32[1]        |
|  3  2  1  0  |  3  |  2  1  0  |
|--------------|-----|-----------|
|           x  |  z  |        y  |
+--------------------------------+
```

The data shape's two `u32` value encodes three values where 4 bytes are dedicated to the primary `x` dimension, 3 bytes dedicated to the secondary `y` dimension and only a single byte dedicated to the tertiary `z` dimension.

### Metadata

A datatable descriptor is 'self-describing'. It contains metadata about its data and its relationship with other datatables.

When using the `TYPE_HINT_INDEX` or `TYPE_HINT_RANGE` the `index_name` refers to the datatable that is being indexed into. This field is required when using these type hints and must not be used on other type hints.

The `related_name` refers to a datatable that has the same exact same type dimensions and values. It implies that the two datatables are connected, the values at the same indices are part of the same record. Think [Structure of Arrays](https://en.wikipedia.org/wiki/AoS_and_SoA#Structure_of_arrays).

## License
