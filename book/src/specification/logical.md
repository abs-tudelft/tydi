Logical stream specification
============================

A Tydi logical stream consists of a bundle of one or more physical streams
and an group of unsynchronized signals, parameterized by way of a logical
stream type. The logical stream type, in turn, represents some representation
of an abstract data type (in the same way that for instance `std::vector` and
`std::list` both represent a sequence in C++). Conceptually, the resulting
logical stream can then be used to stream instances of its corresponding
abstract data type from the source to the sink. The sink can only receive or
operate on one instance at a time and only in sequence (hence the name
"stream"), but the logical stream type can be designed such that the sink as
random-access capability within the current instance.

Logical stream type structure
-----------------------------

The logical stream type is defined recursively by means of the nodes. Two
classes of nodes are defined: stream-manipulating nodes, and
element-manipulating nodes. Only one stream-manipulating node exists
(\\(\textup{Stream}\\)), which is defined in the first subsection. The
remainder of the subsections describe the element-manipulating nodes.

### Stream

The stream node is used as the root node for specifying a logical stream. It is
defined as \\(\textup{Stream}<T_d, n, d, f, c, r, T_u>\\), where:

 - \\(T_d\\) is any logical stream type other than \\(\textup{Stream}\\)
   representing the data type carried by the logical stream;
 - \\(n\\) is a positive real number used to specify the minimum number of
   elements that should be transferrable on the child stream per element in
   the parent stream without the child stream becoming the bottleneck;
 - \\(d\\) is a nonnegative integer specifying the dimensionality of the
   child stream with respect to the parent stream;
 - \\(f\\) (flatten) is a boolean specifying whether the dimensionality
   information for the child stream is carried through its `last` lanes
   (0), or is encoded implicitly in the parent stream (1);
 - \\(c\\) is the complexity number for the physical stream interface, as
   defined in the physical stream specification;
 - \\(r\\) (reverse/response) is a boolean specifying whether the physical
   acts as a response channel to the parent physical stream (1) or carries
   complementary data in the same direction (0);
 - \\(T_u\\) is a logical stream type consisting of only element-manipulating
   nodes, representing the user data carried by this physical stream.

As the \\(\textup{Stream}\\) node carries many parameters and can therefore be
hard to read, the following abbreviations are used within this specification:

 - \\(\textup{New}<T_d, n, c, T_u> \rightarrow \textup{Stream}<T_d, n, 0, 0, c, 0, T_u>\\)
 - \\(\textup{Dim}<T_d, n, c, T_u> \rightarrow \textup{Stream}<T_d, n, 1, 0, c, 0, T_u>\\)
 - \\(\textup{Flat}<T_d, n, c, T_u> \rightarrow \textup{Stream}<T_d, n, 0, 1, c, 0, T_u>\\)
 - \\(\textup{Rev}<T_d, n, c, T_u> \rightarrow \textup{Stream}<T_d, n, 0, 0, c, 1, T_u>\\)

For the above abbreviations, the following defaults apply in addition:

 - \\(T_u = \textup{Null}\\);
 - \\(c =\\) complexity of parent stream (cannot be omitted if there is no
   parent);
 - \\(n = 1\\).

TODO: the flatten flag can't describe everything; may need to use recursive
nodes for the dimension anyway, or at least need yet *another* parameter...

### Null
### Bits
### Group
### Union
