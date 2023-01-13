Untitled Data Format - Command Line Interface
=============================================

Command line interface for manipulating UDF files.

New
---

```
USAGE:
    udf-cli.exe new [OPTIONS] <file>

ARGS:
    <file>    The UDF file

OPTIONS:
    -h, --help            Print help information
        --id [<id>...]    The identifier
```

This tool creates a new empty UDF file at the given path.

Any file at the destination is overwritten.

Validate
--------

```
USAGE:
    udf-cli.exe validate [OPTIONS] <file>

ARGS:
    <file>    The UDF file

OPTIONS:
    -h, --help       Print help information
        --verbose    Verbose output
```

Validates the correctness of the Datasets contained within.

The implementation is incomplete.

Print
-----

```
USAGE:
    udf-cli.exe print [OPTIONS] <file> [--] [path]

ARGS:
    <file>    The UDF file
    <path>    Path to the dataset

OPTIONS:
    -f, --format [<format>...]
            Format option: one of hex, flat, array (default array)

        --file-offset [<file_offset>...]
            File offset to the root dataset

    -h, --help
            Print help information

        --line-width [<line_width>...]
            Sets the line width for the purpose of inserting line breaks (default 75)

    -p, --print-array
            Print the array contents

        --verbose
            Verbose output
```

Prints the content of a Dataset or Datatable.

Export
------

```
USAGE:
    udf-cli.exe export [OPTIONS] <file> <path> <output>

ARGS:
    <file>      The UDF file
    <path>      Path to the dataset
    <output>    Output path

OPTIONS:
    -f, --format [<format>...]              Format option: one of raw, npy (default raw)
        --file-offset [<file_offset>...]    File offset to the root dataset
    -h, --help                              Print help information
        --verbose
```

Exports a Dataset or Datatable.

Import
------

```
USAGE:
    udf-cli.exe import [OPTIONS] <file> <import>

ARGS:
    <file>      The UDF file
    <import>    Path to import file describing the dataset

OPTIONS:
        --create-new    Create a new UDF file instead of updating an existing UDF file
    -h, --help          Print help information
        --set-root      Set the imported dataset as the root dataset
        --verbose       Verbose output
```

This tool imports a Dataset into the udf file. The import file structure is an INI file with an example:

```ini
Id=ID

[Datatable]
TypeInfo=f32:1d
Shape=10x2
Source=npy
FilePath=Datatable.npy
IndexName=OtherDatatable
RelatedName=RelatedDatatable
```

The fields before the section are properties about the Dataset:

* `Id` (optional): is the identifier to be assigned to the Dataset. It is at most 4 ascii characters. When missing the default identifier (empty string) is used instead.

Following that there's an ini section for each Datatable, the name of the Datatable is the section's name with the following properties:

* `TypeInfo` (required): is the type info string, containing the primitive type, dimensions and optional type hint. Examples are `u32:1d:index`, `f32:1d:coord`, `f32:scalar`, `?:1d:json`.

* `Shape` (required): is the shape of the data. Up to 3 dimensions separated by `x`. Examples are `scalar`, `50`, `10x2`, `4x4x3`.

* `Source` (required): specifies how the raw data is specified. It allows these values:

  - `zero`: The binary data is just zeroes.
  - `raw`: The binary data is read directly from the file.
  - `npy`: The binary data is parsed from an [NPY](https://numpy.org/doc/stable/reference/generated/numpy.lib.format.html#module-numpy.lib.format) file.
  - `parse`: The data is parsed from a text file. All characters except `[+\-0-9a-zA-Z.]` are interpreted as spaces and the result is split by whitespace and parsed as the requested primitive type.

* `FilePath` (required*): When `Source` specifies that it reads from a file, this attribute specifies which file the data is sourced from. If the path is relative it is relative from the ini file.

* `IndexName` (optional): specifies the index name.

* `RelatedName` (optional): specifies the related name.

The resulting Dataset is created and written to the UDF file, the final file offset is printed to stdout (and nothing else).

Set root
--------

```
USAGE:
    udf-cli.exe set-root <file> <file-offset>

ARGS:
    <file>           The UDF file
    <file-offset>    The file offset to assign

OPTIONS:
    -h, --help    Print help information
```

This is a dangerous operation that can corrupt the UDF file.
Prefer the `--set-root` option when importing a dataset.

The old root dataset's file offset is printed.
