# Untitled Data Format Specification

## Introduction

## Disk structure

All values are stored in little endian.

## File header

The UDF file starts with a fixed size header of `0x40` bytes:

| Offset | Size   | Name        | Type         | Description
|--------|--------|-------------|--------------|------------
| `0x0`  | `0x4`  | magic       | `[u8;4]`    | File signature, must be ASCII "UDF0": `0x55 0x44 0x46 0x30`. Later revisions may increment the last digit but UDF files will always start with ASCII "UDF".
| `0x4`  | `0x4`  | id          | `[u8;4]`    | Identifier. Helpful for quickly identifying if this UDF file is produced for you intentionally, or if it's just an arbitrary UDF file. This is not a guarantee that the file is according to expectations. Can be zero.
| `0x8`  | `0x8`  | next        | `u64`        | Byte offset to end of file where new datasets can be appended. Readers ignore this field. No dataset file offsets may exceed this threshold.
| `0x10` | `0x10` | root        | `FileOffset` | Location of the root dataset. May be zero.
| `0x20` | `0x20` | reserved    | `[u64;4]`   | Reserved for future use, must be zero.

## File offset

File offsets are used to locate datasets in the UDF file.

| Offset | Size   | Name   | Type  | Description
|--------|--------|--------|-------|------------
| `0x0`  | `0x8`  | offset | `u64` | Byte offset from the start of the file. Must be a multiple of 16.
| `0x8`  | `0x8`  | size   | `u64` | Size in bytes. Must be a multiple of 16.

## Dataset

A dataset is created out of three components: a header, a list of datatable entries followed by storage for the datatable data contents.

The dataset header has a fixed size of `0x20` bytes:

| Offset | Size   | Name         | Type        | Description
|--------|--------|--------------|-------------|------------
| `0x0`  | `0x4`  | check        | `u32`       | Special check value for increasing confidence that this is really a dataset and not some arbitrary bytes.
| `0x4`  | `0x4`  | csum_header  | `u32`       | Header checksum. May be zero.
| `0x8`  | `0x4`  | csum_storage | `u32`       | Storage checksum. May be zero.
| `0xC`  | `0x1`  | len          | `u8`        | Number of datatable entries following this header.
| `0xD`  | `0x1`  | pad_len      | `u8`        | Must be zero.
| `0xE`  | `0x1`  | max_len      | `u8`        | Total number of datatable entries following this header. Can be used to reserve capacity for additional datatable entries. Ignored if its value is less than len.
| `0xF`  | `0x1`  | pad_max_len  | `u8`        | Must be zero.
| `0x10` | `0x4`  | id           | `[u8;4]`    | Identifier. Helpful for quickly identifying the intended structure of this dataset. This is not a guarantee that the dataset is according to expectations. Can be zero.
| `0x14` | `0xC`  | reserved     | `[u32;3]`   | Reserved for future use, must be zero.

The dataset header is immediately followed by `max(header.len, header.max_len)` number of datatable entries. All entries after `header.len` should be ignored, they are reserve capacity.

The datatable entry has a fixed size of `0x30` bytes:

| Offset | Size   | Name          | Type        | Description
|--------|--------|---------------|-------------|------------
| `0x0`  | `0x4`  | key_name      | `u32`       | The hashed key name of this entry.
| `0x4`  | `0x2`  | type_info     | `u16`       | Type information about this datatable's contents.
| `0x6`  | `0x2`  | compress_info | `u16`       | Non-zero if any compression is applied to the data.
| `0x8`  | `0x4`  | mem_start     | `u32`       | See below.
| `0xC`  | `0x4`  | mem_end       | `u32`       | See below.
| `0x10` | `0x4`  | data_size     | `u32`       | Size of the data (after compression) in bytes.
| `0x14` | `0x8`  | data_shape    | `[u32;2]`   | Shape of the data.
| `0x1C` | `0x4`  | index_name    | `u32`       | When an index type hint is used this specifies which datatable the indices go into. Otherwise 0.
| `0x20` | `0x4`  | related_name  | `u32`       | Struct of Arrays (SoA) indicates related datatable.
| `0x24` | `0xC`  | reserved      | `[u32;3]`   | Reserved for future use, must be zero.

The `mem_start` and `mem_end` fields are not in bytes, but rather in 'blocks' of 8 bytes. Multiply these values by 8 to get the byte offset. `mem_end` must be larger or equal to `mem_start`. These offsets start _after_ the datatable entries (NOT! the start of the dataset, this simplifies building datasets). Eg. `mem_start` of `0` starts their data as the first byte after the datatable entries.

This implies that datasets have a limit of `32 GiB` and each datatable has a limit of `4 GiB`. If this is insufficient consider splitting the data over multiple datasets.

## Names

Datatable names are stored as hashed strings to keep things simple. A special 'names datatable' is included which maps the hashed strings back to their original string. This implies that the specifics of the hash is not important (and can really be any assignment) as the names datatable is used to reverse to hash.

## Metadata

A dataset is 'self-describing'. It contains metadata about its data in `type_info` and its relationship with other datatables.
