Physical stream specification
=============================

A Tydi physical stream carries a stream of elements, dimensionality information
for said elements, and (optionally) user-defined transfer information from a
source to a sink. The contents of these three groups and to some extent the way
in which they are transferred are defined in a parameterized way, to provide
the necessary flexibility for the higher levels of this specification, and to
give hardware developers the freedom to optimize their designs. Based on these
parameters, this specification defines which signals must exist on the
interface, and what the significance of these signals is.

Parameters
----------

A physical stream interface has five parameters, described in the following
subsections.

> Note that we define the parameters on the interfaces of the source and the
> sink, not on the stream between them! This is because the complexity
> parameter need not be the same on the source and sink. The others do.

### Element content (E)

This describes the physical layout of the elementary data being transferred by
the stream. It is an emptyable, ordered set of named fields, where each field
has a name and a bit count.

> The element type can be empty to make a "null" stream. Such streams can still
> be useful, as physical streams also carry metadata.

The name of each field is a non-empty, case-insensitive identifier consisting
of letters, numbers, and/or underscores. It cannot start or end with an
underscore, nor can it start with a number. The name must be unique within the
set of named fields.

> The identifier is case-insensitive because compatibility with VHDL is
> desired. It is illegal to start or end with an underscore to prevent
> confusion when the name is prefixed to form the signal name, and (again) for
> compatibility with VHDL.

The bit count cannot be zero.

> A lot of synthesis tools out there do not handle null ranges very well.

The letter \\(E\\) is used as a shorthand for the total number of bits in the
element; that is, the sum of the field bit count over all fields in the
element.

### Number of element lanes (N)

The data described by the element content is physically replicated this many
times, allowing multiple elements to be transferred per stream handshake.
\\(N\\) must be an integer greater than or equal to one.

### Dimensionality (D)

This parameter specifies how many `last` bits are needed to represent the data.
It must be an integer greater than or equal to zero.

> Intuitively, each list nesting level adds one to this number. For instance,
> to stream two-dimensional lists, two last bits will be needed: one to mark
> the boundaries of the inner list, and one to mark the boundary of each
> two-dimensional list.

### Complexity (C)

This parameter specifies what guarantees the source makes about how elements
are transferred. Equivalently, it specifies how many assumptions the sink can
make.

| Higher C                        | Lower C                        |
|---------------------------------|--------------------------------|
| Fewer guarantees made by source | More guarantees made by source |
| Source is easier to implement   | Source is harder to implement  |
| Sink can make fewer assumptions | Sink can make more assumptions |
| Sink is harder to implement     | Sink is easier to implement    |

The complexity levels and signals are defined such that a source with
complexity \\(C_a\\) can be connected to a sink with complexity
\\(C_b \ge C_a\\).

The complexity number is defined as a nonempty, period-separated list of
nonnegative integers. A complexity number is higher than another when the
leftmost integer is greater, and lower when the leftmost integer is lower. If
the leftmost integer is equal, the next integer is checked recursively. If one
complexity number has more entries than another, the shorter number is padded
with zeros on the right.

> That is, it works a bit like a version number. The idea behind this is to
> allow future version of this specification (or any other kind of addition
> later) to insert new complexity levels between the ones defined here. For
> instance, 3 < 3.1 < 3.1.1 < 3.2 < 4.

The restrictions imposed by certain complexity levels are listed in detail
along with the unconditional signal requirements in later sections.
Intuitively, however, the complexity number carries the following significance.

| \\(C\\) | Description                                                                 |
|---------|-----------------------------------------------------------------------------|
| < 8     | All but the highest indexed `last` lane must be all zeros.                  |
| < 7     | The strobe lanes must be all ones.                                          |
| < 6     | The start index must be zero.                                               |
| < 5     | The end index must be \\(N-1\\) for all transfers for which `last` is zero. |
| < 4     | `empty` must be zero if `last` is zero.                                     |
| < 3     | Innermost sequences must be transferred in consecutive cycles.              |
| < 2     | Whole outermost instances must be transferred in consecutive cycles.        |

### User-defined transfer content (U)

This describes the physical layout of the user-defined transfer content of the
stream. It is an emptyable, ordered set of named fields, where each field has a
name and a bit count.

> These fields are like the fields in the element content, but are associated
> to the physical stream transfers rather than the abstract data being
> transferred. They are intended for adding lower-level user-defined protocol
> signals to the stream, similar to the use of the `*USER` signals in AXI4.
> For instance, physical streams could be transferred over a
> network-on-chip-like structure by attaching routing information through this
> method.

The name of each field is a non-empty, case-insensitive identifier consisting
of letters, numbers, and/or underscores. It cannot start or end with an
underscore, nor can it start with a number. The name must be unique within the
set of named fields.

> The identifier is case-insensitive because compatibility with VHDL is
> desired. It is illegal to start or end with an underscore to prevent
> confusion when the name is prefixed to form the signal name, and (again) for
> compatibility with VHDL.

The bit count cannot be zero.

> A lot of synthesis tools out there do not handle null ranges very well.

The letter \\(U\\) is used as a shorthand for the total number of bits in the
transfer content; that is, the sum of the field bit count over all fields in
the transfer content.

