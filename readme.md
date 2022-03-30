Untitled Data Format
====================

Capabilities
------------

This section describes the supported use cases this job format can handle.

### Layer seek time is O(1)

The header contains a table with absolute reference and sizes to all slices.
Each layer is fully self-contained with specified file offset and size.

### Support file mapping

Including 32-bit process support for job files larger than 2 GB.
This is achieved by making each layer start at 64K boundary. All layer data is grouped together with offsets relative to that layer.

### Appendable job file

Trivial to append a new chunk of layer data at the end of the file and write its file offset in the header.

### Layers can be out of order

Layers do not need to be written strictly from first to last.
The header contains a jump table with offsets to each layer.

### Parallel writing layers to disk

Multiple threads can independently append to the job file.
Atomically allocate a chunk and write its file offset in the header.

### Editable job file

Select layers can be replaced on existing job file, allocate a new layer chunk and update the layer offset for a particular slice.
Adding extra layers may not be possible, but removing layer is possible.

### Contour and vector data

Can store both contour and vector data.

### Flexible metadata and type system

Support arbitrary metadata at two places: the stack and slice level.
The datatables support any number of 1D arrays of primitive types or a string, which may contain JSON encoded data.
Type system to easily communicate and validate the expected metadata.

### Support generic viewer

The datatables come with type information to allow a generic viewer to visualize and display information regardless of the specific metadata.

### Prerequisites

Total number of slices and stack datatables must be known before writing the first slice.

Specification
-------------

Command line tool
-----------------

udf.exe overview job.udf
udf.exe fsck job.udf
udf.exe print-table job.udf 0x647f90:0x2a80.Vectors

UdfJobViewer.exe
