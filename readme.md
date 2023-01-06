Untitled Data Format
====================

UDF is a simple, self-describing, structured binary file format for storing arrays and their relationships intended to store scientific and/or industrial data.

It is intended as a light-weight alternative to [HDF](https://en.wikipedia.org/wiki/Hierarchical_Data_Format).

A viewer for UDF files can be found [here](https://casualx.github.io/UntitledDataFormat/).

Introduction
------------

The UDF file is centered around the concept of a directed graph of datasets. A dataset is an unordered list of datatables and their properties. A datatable 

Capabilities
------------

### Simple file format

The format on disk is simple enough to make it feasible to create custom implementations optimized for your project.

### Extensible data format

Datasets contain a dynamic number of datatables, making it possible to add additional datatables without interfering with existing applications.

### Supports random access

Due to its use of absolute _File offsets_ to reference data it is efficient to read only the parts of the file that are needed, skipping over everything else. This makes it feasible to read file contents over a network connection (eg. HTTP Range requests, see the viewer samples for a demo).

### Editable, appendable file format

It is trivial to append new data to the end of the file and rewriting offsets to point to this new data. The previous data remains but is inaccessible (due to not being referenced). The unreferenced datasets can later be cleaned up with a separate garbage collection pass.

### Datasets can be out of order

_Datasets_ do not have to be written in a specific order. They are referenced by absolute _File offsets_ making generating the data flexible.

On top of that the _Datasets_ can be written to the file in parallel with minimal coordination.

### Robust against errors

The file format contains multiple defences to mitigate data loss from corruption. It is encouraged to append new datasets keeping old data around for a while. These old datasets can be recovered easily or garbage collected over time. Existing data is covered by checksums to verify integrity of only the datasets of interest.

### Supports file mapping

The UDF file can be file mapped for convenient access to its structures. The structures and data are aligned to allow zero-copy access. Each _Dataset_ is self-contained making it feasible to map only the data you're interested in.

### Self-describing data format

Extensive system to describe the datatables and its relationships.

Each datatable contains its primitive type (eg. `u8`, `i32`, `f64`, ...), dimensions and shape, a type hint to help interpret the data.

Relationships between datatables can take two forms: an index relationship (the datatable is an index or range into another datatable) and a related relationship (the datatable represents columns of the same structure).

### Supports generic data viewer

The datatables come with type information to allow a generic viewer to visualize and display information regardless of the specific metadata.

Specification
-------------

Detailed specification can be found [here](specification.md).

Command line tool
-----------------

TODO!
