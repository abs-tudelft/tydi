Logical streams
===============

A Tydi logical stream consists of a bundle of physical streams and an group of
unsynchronized signals, parameterized by way of a logical stream type. The
logical stream type, in turn, represents some representation of an abstract
data type (in the same way that for instance `std::vector` and `std::list` both
represent a sequence in C++). Conceptually, the resulting logical stream can
then be used to stream instances of its corresponding abstract data type from
the source to the sink. The sink can only receive or operate on one instance at
a time and only in sequence (hence the name "stream"), but the logical stream
type can be designed such that the sink as random-access capability within the
current instance.

Logical stream type structure
-----------------------------

The logical stream type is defined recursively by means of the nodes. Two
classes of nodes are defined: stream-manipulating nodes, and
element-manipulating nodes. Only one stream-manipulating node exists
(\\(\mathrm{Stream}\\)), which is defined in the first subsection. The
remainder of the subsections describe the element-manipulating nodes.

### Stream

The \\(\mathrm{Stream}\\) node is used to define a new physical stream. It is
defined as \\(\mathrm{Stream}(T_d, n, d, s, c, r, T_u)\\), where:

 - \\(T_d\\) is any logical stream type representing the data type carried by
   the logical stream.
 - \\(n\\) is a positive real number used to specify the minimum number of
   elements that should be transferrable on the child stream per element in
   the parent stream without the child stream becoming the bottleneck (with no
   parent, it is the initial value).
 - \\(d\\) is a nonnegative integer specifying the dimensionality of the
   child stream with respect to the parent stream (with no parent, it is the
   initial value).
 - \\(s\\) specifies the synchronicity of the \\(d\\)-dimensional elements in
   the child stream with respect to the elements in the parent stream. It must
   be one of:
    - \\(\mathrm{Sync}\\), indicating that there is a one-to-one relation
      between the parent and child elements, and the dimensionality information
      of the parent stream is redundantly carried by the child stream as well;
    - \\(\mathrm{Flatten}\\), indicating that there is a one-to-one relation
      between the parent and child elements, and the dimensionality information
      of the parent stream is omitted in the child stream;
    - \\(\mathrm{Desync}\\), indicating that there is no relation between the
      parent and child elements.
   \\(s\\) carries no significance if there is no parent stream.
 - \\(c\\) is the complexity number for the physical stream interface, as
   defined in the physical stream specification.
 - \\(r\\) (reverse/response) specifies the direction of the stream. It must be
   one of:
    - \\(\mathrm{Forward}\\), indicating that the child stream carries data
      complementary to the data carried by the parent stream, in the same
      direction;
    - \\(\mathrm{Reverse}\\), indicating that the child stream acts as a
      response channel for the parent stream.
   If there is no parent stream, \\(r\\) specifies the direction with respect
   to the natural direction of the stream (source to sink).
 - \\(T_u\\) is a logical stream type consisting of only element-manipulating
   nodes, representing the user data carried by this physical stream.

> As the \\(\mathrm{Stream}\\) node carries many parameters and can therefore
> be hard to read, some abbreviations are in order:
> 
>  - \\(\mathrm{Dim}(T_d, n, c, T_u) \ \rightarrow\ \mathrm{Stream}(T_d, n, 1, \mathrm{Sync},    c, \mathrm{Forward}, T_u)\\)
>  - \\(\mathrm{New}(T_d, n, c, T_u) \ \rightarrow\ \mathrm{Stream}(T_d, n, 0, \mathrm{Sync},    c, \mathrm{Forward}, T_u)\\)
>  - \\(\mathrm{Des}(T_d, n, c, T_u) \ \rightarrow\ \mathrm{Stream}(T_d, n, 0, \mathrm{Desync},  c, \mathrm{Forward}, T_u)\\)
>  - \\(\mathrm{Flat}(T_d, n, c, T_u)\ \rightarrow\ \mathrm{Stream}(T_d, n, 0, \mathrm{Flatten}, c, \mathrm{Forward}, T_u)\\)
>  - \\(\mathrm{Rev}(T_d, n, c, T_u) \ \rightarrow\ \mathrm{Stream}(T_d, n, 0, \mathrm{Sync},    c, \mathrm{Reverse}, T_u)\\)
> 
> For the above abbreviations, the following defaults apply in addition:
> 
>  - \\(T_u = \mathrm{Null}\\);
>  - \\(c =\\) complexity of parent stream (cannot be omitted if there is no
>    parent);
>  - \\(n = 1\\).

### Null

The \\(\mathrm{Null}\\) leaf node indicates the transferrence of one-valued
data: its only valid value is \\(\varnothing\\) (null).

> This is useful primarily when used within a \\(\mathrm{Union}\\), where the
> data type itself carries information. Furthermore, \\(\mathrm{Stream}\\)
> nodes carrying a \\(\mathrm{Null}\\) will always result in a physical stream
> (whereas \\(\mathrm{Stream}\\) nodes carrying only other streams will be
> "optimized out"), which may still carry dimensionality information, user
> data, or even just transfer timing information.

### Bits

The \\(\mathrm{Bits}\\) leaf node, defined as \\(\mathrm{Bits}(b)\\) indicates
the transferrence of \\(2^b\\)-valued data carried by means of a group of
\\(b\\) bits, where \\(b\\) is a positive integer.

> Tydi does not specify how the bits should be interpreted: such interpretation
> is explicitly out of scope. Where necessary for examples, this specification
> will usually use unsigned binary numbers or ASCII characters expressed using
> 8 bits.

### Group

The \\(\mathrm{Group}\\) node acts as a product type for composition. It is
defined as \\(\mathrm{Group}(N_1: T_1, N_2: T_2, ..., N_n: T_n)\\), where:

 - \\(N_i\\) is a string consisting of letters, numbers, and/or underscores,
   representing the name of the \\(i\\)th field;
 - \\(T_i\\) is the logical stream type for the \\(i\\)th field;
 - \\(n\\) is a nonnegative integer, representing the number of fields.

> Intuitively, each instance of type \\(\mathrm{Group}(...)\\) consists of
> instances of *all* the contained types. This corresponds to a `struct` or
> `record` in most programming languages.

The names cannot start or end with an underscore.

The names cannot start with a digit.

The names cannot be empty.

The names must be case-insensitively unique within the group.

> The above requirements on the name mirror the requirements on the field names
> for the physical streams.

### Union

The \\(\mathrm{Union}\\) node acts as a sum type for exclusive disjunction. It
is defined as \\(\mathrm{Union}(N_1: T_1, N_2: T_2, ..., N_n: T_n)\\), where:

 - \\(N_i\\) is a string consisting of letters, numbers, and/or underscores,
   representing the name of the \\(i\\)th field;
 - \\(T_i\\) is the logical stream type for the \\(i\\)th field;
 - \\(n\\) is a positive integer, representing the number of fields.

> Intuitively, each instance of type \\(\mathrm{Union}(...)\\) consists of
> an instance of *one* the contained types, as well as a tag indicating which
> field that instance belongs to. This corresponds to a `union` or `enum` in
> most programming languages. Mathematically, Tydi unions represent tagged
> unions.

The names cannot start or end with an underscore.

The names cannot start with a digit.

The names cannot be empty.

The names must be case-insensitively unique within the group.

> The above requirements on the name mirror the requirements on the field names
> for the physical streams.
