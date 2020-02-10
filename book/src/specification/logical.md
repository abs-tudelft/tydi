Logical streams
===============

A Tydi logical stream consists of a bundle of physical streams and a group of
user-defined signals, parameterized by way of a logical stream type. The
logical stream type, in turn, represents some representation of an abstract
data type (in the same way that for instance `std::vector` and `std::list` both
represent a sequence in C++). Conceptually, the resulting logical stream can
then be used to stream instances of its corresponding abstract data type from
the source to the sink. The sink can only receive or operate on one instance at
a time and only in sequence (hence the name "stream"), but the logical stream
type can be designed such that the sink as random-access capability within the
current instance.

Because of the degree of flexibility in specifying Tydi streams, much of this
specification consists of the formal definitions of the structures needed to
describe a Tydi stream, and operations thereupon. The resulting constraints on
the signal lists and behavior of the signals are then described based on these.

Logical stream type structure
-----------------------------

The logical stream type is defined recursively by means of the nodes. Two
classes of nodes are defined: stream-manipulating nodes, and
element-manipulating nodes. Only one stream-manipulating node exists
(\\(\mathrm{Stream}\\)), which is defined in the first subsection. The
remainder of the subsections describe the element-manipulating nodes.

### Stream

The \\(\mathrm{Stream}\\) node is used to define a new physical stream. It is
defined as \\(\mathrm{Stream}(T_d, n, d, s, c, r, T_u, x)\\), where:

 - \\(T_d\\) is any logical stream type;

   > It represents the data type carried by the logical stream. This may
   > include additional \\(\textrm{Stream}\\) nodes to handle cases such as
   > structures containing sequences, handled in Tydi by transferring the
   > two different dimensions over two different physical streams, as well as
   > to encode request-response patterns.

 - \\(n\\) is a positive real number;

   > It represents the minimum number of elements that should be transferrable
   > on the child stream per element in the parent stream without the child
   > stream becoming the bottleneck. As we'll see later, the \\(N\\) parameter
   > for the resulting physical stream equals \\(\left\lceil\prod n\right\rceil\\)
   > for all ancestral \\(\textrm{Stream}\\) nodes, including this node.

 - \\(d\\) is a nonnegative integer;

   > It represents the dimensionality of the child stream with respect to the
   > parent stream. As we'll see later, the \\(D\\) parameter for the resulting
   > physical stream equals \\(\sum d\\) for all ancestral
   > \\(\textrm{Stream}\\) nodes, including this node, up to and including the
   > closest ancestor (or self) for which \\(s = \mathrm{Flatten}\\).

 - \\(s\\) must be one of \\(\mathrm{Sync}\\), \\(\mathrm{Flatten}\\),
   \\(\mathrm{Desync}\\), or \\(\mathrm{FlatDesync}\\);

   > This parameter specifies the relation between the dimensionality
   > information of the parent stream and the child stream. The most common
   > relation is \\(\mathrm{Sync}\\), which enforces that for each element in
   > the parent stream, there must be one \\(d\\)-dimensional sequence in the
   > child stream (if \\(d\\) is 0, that just means one element). Furthermore,
   > it means that the `last` flags from the parent stream are repeated in the
   > child stream.  This repetition is technically redundant; since the sink
   > will have access to both streams, it only needs this information once to
   > decode the data.
   >
   > If this redundant repetition is not necessary for the sink or hard to
   > generate for a source, \\(\mathrm{Flatten}\\) may be used instead. It is
   > defined in the same way, except the redundant information is not repeated
   > for the child stream.
   >
   > Consider for example the data `[(1, [2, 3]), (4, [5])], [(6, [7])]`. If
   > encoded as \\(\mathrm{Stream}(d=1, T_d=\textrm{Group}(\textrm{Bits}(8), \mathrm{Stream}(d=1, s=\mathrm{Sync}, ...)), ...)\\),
   > the first physical stream will transfer `[1, 4], [6]`, and the second
   > physical stream will transfer `[[2, 3], [5]], [[7]]`. If
   > \\(\mathrm{Flatten}\\) is used for the inner \\(\textrm{Stream}\\)
   > instead, the data transferred by the second physical stream reduces to
   > `[2, 3], [5], [7]`; the structure of the outer sequence is implied by the
   > structure transferred by the first physical stream.
   >
   > \\(\mathrm{Desync}\\) may be used if the relation between the elements in
   > the child and parent stream is dependent on context rather than the `last`
   > flags in either stream. For example, for the type
   > \\(\mathrm{Stream}(d=1, T_d=\textrm{Group}(\textrm{Bits}(8), \mathrm{Stream}(s=\mathrm{Desync}, ...)), ...)\\),
   > if the first stream carries `[1, 4], [6]` as before, the child stream may
   > carry anything of the form `[...], [...]`. That is, the dimensionality
   > information (if any) must still match, but the number of elements in the
   > innermost sequence is unrelated or only related by some user-defined
   > context.
   >
   > \\(\mathrm{FlatDesync}\\), finally, does the same thing as
   > \\(\mathrm{Desync}\\), but also strips the dimensionality information from
   > the parent. This means there the relation between the two streams, if any,
   > is fully user-defined.

 - \\(c\\) is a complexity number, as defined in the physical stream
   specification;

   > It specifies the complexity for the corresponding physical stream.

 - \\(r\\) must be one of \\(\mathrm{Forward}\\) or \\(\mathrm{Reverse}\\);

   > It specifies the direction of the child stream with respect to the parent
   > stream, or if there is no parent stream, with respect to the natural
   > direction of the logical stream (source to sink). \\(\mathrm{Forward}\\)
   > indicates that the child stream flows in the same direction as its parent,
   > complementing the data of its parent in some way. Conversely,
   > \\(\mathrm{Reverse}\\) indicates that the child stream acts as a response
   > channel for the parent stream.

 - \\(T_u\\) is a logical stream type consisting of only element-manipulating
   nodes.

   > It represents the `user` signals of the physical stream; that is,
   > transfer-oriented data (versus element-oriented data).
   >
   > Note that while \\(T_u\\) is called a "logical stream type" for
   > consistency, the \\(\textrm{Stream}\\) node being illegal implies that no
   > additional streams can be specified within this type.

 - \\(x\\) is a boolean.

   > It specifies whether the stream carries "extra" information beyond the
   > `data` and `user` signal payloads. \\(x\\) is normally false, which
   > implies that the \\(\textrm{Stream}\\) node will not result in a physical
   > stream if both its `data` and `user` signals would be empty according to
   > the rest of this specification; it is effectively optimized away. Setting
   > \\(x\\) to true simply overrides this behavior.
   >
   > This optimization handles a fairly common case when the logical stream
   > type is recursively generated by tooling outside the scope of this layer
   > of the specification. It arises for instance for nested lists, which may
   > result in something of the form
   > \\(\textrm{Stream}(d=1, T_d=\textrm{Stream}(d=1, s=\textrm{Sync}, T_d=...))\\).
   > The outer stream is not necessary here; all the dimensionality information
   > is carried by the inner stream.
   >
   > It could be argued that the above example should be reduced to
   > \\(\textrm{Stream}(d=2, s=\textrm{Sync}, T_d=...)\\) by said external
   > tool. Indeed, this description is equivalent. This reduction is however
   > very difficult to define or specify in a way that it handles all cases
   > intuitively, hence this more pragmatic solution was chosen.

> The function is applied to the data type of \\(\textrm{Stream}\\) nodes by
> \\(\textrm{Split}()\\) to determine whether the \\(\textrm{Stream}\\) node
> carries information, or only serves to augment the contained streams. For
> instance, \\(\textrm{Stream}(d=1, T_d=\textrm{Stream}(d=1, T_d=...)\\)
> is likely to result from constructing the type for two nested lists, but the
> inner stream carries all the required information, so the outer stream can
> be removed.
>
> A stream is said to carry significant data if it has a non-empty `data` or
> `user` signal, if its data type contains a \\(\textrm{Null}\\) that is not
> also a descendent of a descendent \\(\textrm{Stream}\\) node, or if its



> As the \\(\mathrm{Stream}\\) node carries many parameters and can therefore
> be hard to read, some abbreviations are in order:
> 
>  - \\(\mathrm{Dim}(T_d, n, c, T_u) \ \rightarrow\ \mathrm{Stream}(T_d, n, 1, \mathrm{Sync},    c, \mathrm{Forward}, T_u, 0)\\)
>  - \\(\mathrm{New}(T_d, n, c, T_u) \ \rightarrow\ \mathrm{Stream}(T_d, n, 0, \mathrm{Sync},    c, \mathrm{Forward}, T_u, 0)\\)
>  - \\(\mathrm{Des}(T_d, n, c, T_u) \ \rightarrow\ \mathrm{Stream}(T_d, n, 0, \mathrm{Desync},  c, \mathrm{Forward}, T_u, 0)\\)
>  - \\(\mathrm{Flat}(T_d, n, c, T_u)\ \rightarrow\ \mathrm{Stream}(T_d, n, 0, \mathrm{Flatten}, c, \mathrm{Forward}, T_u, 0)\\)
>  - \\(\mathrm{Rev}(T_d, n, c, T_u) \ \rightarrow\ \mathrm{Stream}(T_d, n, 0, \mathrm{Sync},    c, \mathrm{Reverse}, T_u, 0)\\)
> 
> For the above abbreviations, the following defaults apply in addition:
> 
>  - \\(T_u = \mathrm{Null}\\);
>  - \\(c =\\) complexity of parent stream (cannot be omitted if there is no
>    parent);
>  - \\(n = 1\\).

### Null

The \\(\mathrm{Null}\\) leaf node indicates the transferrence of one-valued
data: it is only valid value is \\(\varnothing\\) (null).

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

The names cannot contain two or more consecutive underscores.

> Double underscores are used by Tydi as a hierarchy separator when flattening
> structures for the canonical representation. If not for the above rule,
> something of the form
> \\(\textrm{Group}(\textrm{'a'}:\textrm{Group}(\textrm{'b__c'}:...),\textrm{'a__b'}:\textrm{Group}(\textrm{'c'}:...))\\)
> would result in a name conflict: both signals would be named `a__b__c`.

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

The names cannot contain two or more consecutive underscores.

> Double underscores are used by Tydi as a hierarchy separator when flattening
> structures for the canonical representation. If not for the above rule,
> something of the form
> \\(\textrm{Union}(\textrm{'a'}:\textrm{Union}(\textrm{'b__c'}:\textrm{Stream}...),\textrm{'a__b'}:\textrm{Union}(\textrm{'c'}:\textrm{Stream}...))\\)
> would result in a name conflict: both physical streams would be named `a__b__c`.

The names cannot start or end with an underscore.

The names cannot start with a digit.

The names cannot be empty.

The names must be case-insensitively unique within the group.

> The above requirements on the name mirror the requirements on the field names
> for the physical streams.

#### Semantics

When flattened into fields, a \\(\textrm{Union}\\) node consists of:

 - if there are two or more variants, an `"tag"` field of size
   \\(\left\lceil\log_2{n}\right\rceil\\), where \\(n\\) is the number of
   variants; and
 - if any variant carries data in the same stream as the union itself, a
   `"union"` field of size\
   \\(\max_{\textrm{variants}} \sum_{\textrm{subfields}} b_{\textrm{subfield}}\\).

The `"tag"` field is used to represent which variant is being represented, by
means of a zero-based index encoded as an unsigned binary number.

It is illegal for a source to drive the `"tag"` field of a \\(\textrm{Union}\\)
with \\(n\\) variants to \\(n\\) or higher.

> The above would otherwise be possible if \\(n\\) is not a power of two.

The `"union"` field of a \\(\textrm{Union}\\) is used to represent variant
data existing in the same stream as the union itself. It consists of the
LSB-first, LSB-aligned concatenation of the fields of the variant.

> No constraint is placed on the value that the source drives on the pad bits.

The physical streams represented by \\(\textrm{Stream}\\) nodes nested inside
\\(\textrm{Union}\\) variant types behave as if any other variants carried by
the parent stream do not exist, but are otherwise synchronized to the parent
stream as defined by the \\(s\\) parameter of the child stream.

> Consider the following example, keeping \\(x\\) generic for now:
>
> \\[B = \textrm{Group}(x: \textrm{Bits}(2), y: \textrm{Bits}(2))\\\\
> C = \textrm{Stream}(d=1, s=x, T_d=\textrm{Bits}(4))\\\\
> U = \textrm{Union}(\textrm{a} : \textrm{Bits}(3), \textrm{b} : \textrm{Bits}(3), \textrm{c} : C)\\\\
> \textrm{Stream}(d=1, T_d=U)\\]
>
> This results in two physical streams:
>
>  - the unnamed stream carrying the union with \\(D = 1\\), consisting of a
>    two-bit field named `tag` and a four-bit field named `union`; and
>  - a stream named `c`, consisting of a single unnamed four-bit field.
>
> Let's say that we need to represent the data `[a:0, b:(x:1, y:2)], [c:[3, 4, 5], a:6]`.
> The unnamed stream then carries the following four transfers, regardless of
> the value for \\(x\\):
>
> ```text
> [ (tag: "00", union: "-000"),     (1)
>   (tag: "01", union: "1001") ],   (2)
> [ (tag: "10", union: "----"),     (3)
>   (tag: "00", union: "-110") ]    (4)
> ```
>
> In transfer 1 and 4, the MSB of the `union` is don't-care, because variant
> `a` uses only three bits. In transfer 2, `x` is represented by the two LSB
> of the `union` field, while `y` is represented by the MSBs. Finally, in
> transfer 3, all bits of `union` are don't-care, as stream `c` is used to
> carry the data. Note that a `tag` value of `"11"` is illegal in this example.
>
> When \\(x = \textrm{Sync}\\), stream `c` has \\(D = 2\\) and carries the
> following transfers:
>
> ```text
> [              ],   (1)
> [ [ ("0011"),       (2)
>     ("0100"),       (3)
>     ("0101") ] ]    (4)
> ```
>
> Transfer 1 carries an empty outer sequence, corresponding to
> `[a:0, b:(x:1, y:2)]` of the represented data. There are no inner sequences
> because variant `c` never occurs in the first sequence, but the outer
> sequence must still be represented to correctly copy the dimensionality
> information of the parent stream, as mandated by the \\(\textrm{Sync}\\) tag.
> The second outer sequence represents `[c:[3, 4, 5], a:6]`; it carries one
> inner sequence, because variant `c` occurs once. The inner sequence directly
> corresponds to `[3, 4, 5]`.
>
> When \\(x = \textrm{Flatten}\\), the outer sequence is flattened away.
> Therefore, stream `c` has \\(D = 1\\) and carries only the following
> transfers:
>
> ```text
> [ ("0011"),     (1)
>   ("0100"),     (2)
>   ("0101") ]    (3)
> ```
>
> It is then up to the sink to recover the outer dimension if it needs it.
>
> When \\(x = \textrm{Desync}\\), \\(D = 2\\) as before. For the example data,
> the transfers would be exactly the same as for \\(x = \textrm{Sync}\\), but
> the zero-or-more relationship defined by the \\(\textrm{Desync}\\) means that
> a `c` variant in the parent stream may correspond with any number of inner
> sequences on the `c` stream. That does however still mean that the first
> outer sequence must be empty, as follows:
>
> ```text
> [         ],
> [ [ ... ]
>     ...
>   [ ... ] ]
> ```
>
> After all, there are no `c` variants in the first sequence to apply the
> zero-or-more relationship to.
>
> Finally, \\(x = \textrm{FlatDesync}\\) flattens the outer sequence away
> compared to \\(x = \textrm{Desync}\\), allowing for any number of transfers
> to occur on the `c` stream, all corresponding to the `c` variant in transfer
> 3 of the parent stream.
>
> If you're confused as to how a sink is to determine which transfers on the
> `c` stream correspond to which `c` variant in the parent stream, recall that
> the purpose of \\(\textrm{Desync}\\) and \\(\textrm{FlatDesync}\\) is to
> allow the user of this layer of the specification to define this relation as
> they see fit. Usually, a pattern of the form
> \\(\textrm{Group}(\textrm{len}: \textrm{Bits}(...), \textrm{data}: \textrm{Stream}(s=\textrm{Desync}, ...))\\)
> would be used, in which case the length of the encoded inner sequence would
> be transferred using the `union` field of the parent stream.

Type compatibility function
---------------------------

This section defines the function
\\(\mathrm{compatible}(T_{source}, T_{sink}) \rightarrow \textrm{Bool}\\),
where \\(T_{source}\\) and \\(T_{sink}\\) are both logical stream types. The
function returns true if and only if a source of type \\(T_{source}\\) can be
connected to a sink of type \\(T_{sink}\\) without insertion of conversion
logic. The function returns true if and only if one or more of the following
rules match:

 - \\(T_{source} = T_{sink}\\);

 - \\(T_{source} = \mathrm{Stream}(T_d, n, d, s, c, r, T_u, x)\\),
   \\(T_{sink} = \mathrm{Stream}(T'_d, n, d, s, c', r, T_u, x)\\),
   \\(\mathrm{compatible}(T_d, T'_d) = \textrm{true}\\), and \\(c < c'\\);

 - \\(T_{source} = \mathrm{Group}(N_1: T_1, N_2: T_2, ..., N_n: T_n)\\),
   \\(T_{sink} = \mathrm{Group}(N_1: T'_1, N_2: T'_2, ..., N_n: T'_n)\\),
   and \\(\mathrm{compatible}(T_i, T'_i)\\) is true for all
   \\(i \in (1, 2, ..., n)\\); or

 - \\(T_{source} = \mathrm{Union}(N_1: T_1, N_2: T_2, ..., N_n: T_n)\\),
   \\(T_{sink} = \mathrm{Union}(N_1: T'_1, N_2: T'_2, ..., N_n: T'_n)\\),
   and \\(\mathrm{compatible}(T_i, T'_i)\\) is true for all
   \\(i \in (1, 2, ..., n)\\).

The name matching is to be done case sensitively.

> While two streams may be compatible in a case-insensitive language when only
> the case of one or more of the names differ, this would still not hold for
> case-sensitive languages. Therefore, for two streams to be compatible in all
> cases, the matching must be case sensitive.

Null detection function
-----------------------

This section defines the function
\\(\mathrm{isNull}(T_{in}) \rightarrow \textrm{Bool}\\), which returns
true if and only if \\(T_{in}\\) does not result in any signals.

The function returns true if and only if one or more of the following
rules match:

 - \\(T_{in} = \mathrm{Stream}(T_d, n, d, s, c, r, T_u, x)\\),
   \\(\mathrm{isNull}(T_d)\\) is true, \\(\mathrm{isNull}(T_u)\\) is true,
   and \\(x\\) is false;

 - \\(T_{in} = \mathrm{Null}\\);

 - \\(T_{source} = \mathrm{Group}(N_1: T_1, N_2: T_2, ..., N_n: T_n)\\) and
   \\(\mathrm{isNull}(T_i)\\) is true for all \\(i \in (1, 2, ..., n)\\); or

 - \\(T_{source} = \mathrm{Union}(N: T)\\) and \\(\mathrm{isNull}(T)\\) is true.

Split function
--------------

This section defines the function
\\(\mathrm{split}(T_{in}) \rightarrow \textrm{SplitStreams}(T_{signals}, N_1 : T_1, N_2 : T_2, ..., N_n : T_n)\\),
where \\(T_{in}\\) is any logical stream type, \\(T_{signals}\\) is a logical
stream type consisting of only element-manipulating nodes, all \\(T_{1..n}\\)
are \\(\textrm{Stream}\\) nodes with the data and user subtypes consisting
of only element-manipulating nodes, all \\(N\\) are case-insensitively
unique, emptyable strings consisting of letters, numbers, and/or underscores,
not starting or ending in an underscore, and not starting with a digit, and
\\(n\\) is a nonnegative integer.

> Intuitively, it splits a logical stream type up into simplified stream types,
> where \\(T_{signals}\\) contains only the information for the user-defined
> signals, and \\(T_i\\) contains only the information for the physical stream
> with index \\(i\\) and name \\(N_i\\).
>
> The names can be empty, because if there are no \\(Group\\)s or \\(Union\\)s,
> the one existing stream or signal will not have an intrinsic name given by
> the type. Empty strings are represented here with \\(\varnothing\\). Note
> that the names here can include double underscores.

\\(\mathrm{split}(T_{in})\\) is evaluated as follows.

 - If \\(T_{in} = \mathrm{Stream}(T_d, n, d, s, c, r, T_u, x)\\), apply the
   following algorithm.

    - Initialize \\(N\\) and \\(T\\) to empty lists.
    - \\(T_{data} := \mathrm{split}(T_d)\_{signals}\\) (i.e., the
      \\(T_{signals}\\) item of the tuple returned by the \\(\mathrm{split}\\)
      function)
    - If \\(\textrm{isNull}(T_{data})\\) is false, \\(\textrm{isNull}(T_u)\\)
      is false, or \\(x\\) is true:
       - Append \\(\varnothing\\) (an empty name) to the end of \\(N\\).
       - Append \\(\mathrm{Stream}(T_{data}, n, d, s, c, r, T_u, x)\\) to the end
         of \\(T\\).
    - Extend \\(N\\) with \\(\mathrm{split}(T_d)\_{names}\\) (i.e.,
      all stream names returned by the \\(\mathrm{split}\\) function).
    - For all \\(T' \in \mathrm{split}(T_d)\_{streams}\\) (i.e.,
      for all named streams returned by the \\(\mathrm{split}\\) function):
       - Unpack \\(T'\\) into \\(\mathrm{Stream}(T'\_d, n', d', s', c', r', T'\_u, x')\\).
         This is always possible due to the postconditions of
         \\(\textrm{split}\\).
       - If \\(r = \textrm{Reverse}\\), reverse the direction of \\(r'\\).
       - If \\(s = \textrm{Flatten}\\) or \\(s = \textrm{FlatDesync}\\),
         assign \\(s' := \textrm{FlatDesync}\\).
       - If \\(s' \ne \textrm{Flatten}\\) and \\(s \ne \textrm{FlatDesync}\\),
         assign \\(d' := d' + d\\).
       - \\(n' := n' \cdot n\\).
       - Append \\(\mathrm{Stream}(T'\_d, n', d', s', c', r', T'\_u, x')\\) to
         \\(T\\).
    - Return \\(\textrm{SplitStreams}(\textrm{Null}, N_1 : T_1, N_2 : T_2, ..., N_m : T_m)\\).

 - If \\(T_{in} = \textrm{Null}\\) or \\(T_{in} = \textrm{Bits}(b)\\),
   return \\(\textrm{SplitStreams}(T_{in})\\).

 - If \\(T_{in} = \textrm{Group}(N_{g,1} : T_{g,1}, N_{g,2} : T_{g,2}, ..., N_{g,n} : T_{g,n})\\),
   apply the following algorithm.

    - \\(T_{signals} := \textrm{Group}(N_{g,1} : \mathrm{split}(T_{g,1})\_{signals}, ..., N_{g,n} : \mathrm{split}(T_{g,n})\_{signals})\\)
      (i.e., replace the types contained by the group with the
      \\(T_{signals}\\) item of the tuple returned by the \\(\mathrm{split}\\)
      function applied to the original types).
    - Initialize \\(N\\) and \\(T\\) to empty lists.
    - For all \\(i \in 1..n\\):
       - For all \\(\textrm{name} \in \mathrm{split}(T_{g,i})\_{names}\\) (i.e.,
         for all stream names returned by the \\(\mathrm{split}\\) function),
          - If \\(\textrm{name} = \varnothing\\), append \\(N_{g,i}\\) to the end
            of \\(N\\).
          - Otherwise, append the concatenation \\(N_{g,i}\\) and \\(\textrm{name}\\)
            to the end of \\(N\\), separated by a double underscore.
       - Extend \\(T\\) with \\(\mathrm{split}(T_{g,i})\_{streams}\\) (i.e.,
         all named streams returned by the \\(\mathrm{split}\\) function).
    - Return \\(\textrm{SplitStreams}(T_{signals}, N_1 : T_1, N_2 : T_2, ..., N_m : T_m)\\),
      where \\(m = |N| = |T|\\).

 - If \\(T_{in} = \textrm{Union}(N_{u,1} : T_{u,1}, N_{u,2} : T_{u,2}, ..., N_{u,n} : T_{u,n})\\),
   apply the following algorithm.

    - \\(T_{signals} := \textrm{Union}(N_{u,1} : \mathrm{split}(T_{u,1})\_{signals}, ..., N_{u,n} : \mathrm{split}(T_{u,n})\_{signals})\\)
      (i.e., replace the types contained by the group with the
      \\(T_{signals}\\) item of the tuple returned by the \\(\mathrm{split}\\)
      function applied to the original types).
    - Initialize \\(N\\) and \\(T\\) to empty lists.
    - For all \\(i \in 1..n\\):
       - For all \\(\textrm{name} \in \mathrm{split}(T_{u,i})\_{names}\\) (i.e.,
         for all stream names returned by the \\(\mathrm{split}\\) function),
          - If \\(\textrm{name} = \varnothing\\), append \\(N_{u,i}\\) to the end
            of \\(N\\).
          - Otherwise, append the concatenation \\(N_{u,i}\\) and \\(\textrm{name}\\)
            to the end of \\(N\\), separated by a double underscore.
       - Extend \\(T\\) with \\(\mathrm{split}(T_{u,i})\_{streams}\\) (i.e.,
         all named streams returned by the \\(\mathrm{split}\\) function).
    - Return \\(\textrm{SplitStreams}(T_{signals}, N_1 : T_1, N_2 : T_2, ..., N_m : T_m)\\),
      where \\(m = |N| = |T|\\).

> Note that the algorithm for \\(\textrm{Group}\\) and \\(\textrm{Union}\\) is
> the same, aside from returning \\(\textrm{Group}\\) vs \\(\textrm{Union}\\)
> nodes for \\(T_{signals}\\).

Field conversion function
-------------------------

This section defines the function
\\(\mathrm{fields}(T_{in}) \rightarrow \textrm{Fields}(N_1 : b_1, N_2 : b_2, ..., N_n : b_n)\\),
where \\(T_{in}\\) is any logical stream type, all \\(N\\) are
case-insensitively unique, emptyable strings consisting of letters, numbers,
and/or underscores, not starting or ending in an underscore, and not starting
with a digit, all \\(b\\) are positive integers, and \\(n\\) is a nonnegative
integer.

> Intuitively, this function flattens a logical stream type consisting of
> \\(\textrm{Null}\\), \\(\textrm{Bits}\\), \\(\textrm{Group}\\) and
> \\(\textrm{Union}\\) nodes into a list of named bitfields. It is used for
> constructing the data and user field lists of the physical streams and the
> asynchronous signal list from the logical stream types preprocessed by
> \\(\textrm{split}()\\). This function is normally only applied to logical
> stream types that only carry element-manipulating nodes.
>
> Note that the definition for \\(\textrm{Fields}) here matches the definition
> in the physical stream specification.
>
> The names can be empty, because if there are no \\(Group\\)s or \\(Union\\)s,
> the one existing stream or signal will not have an intrinsic name given by
> the type. Empty strings are represented here with \\(\varnothing\\). Note
> that the names here can include double underscores.

\\(\mathrm{fields}(T_{in})\\) is evaluated as follows.

 - If \\(T_{in} = \textrm{Null}\\) or
   \\(T_{in} = \mathrm{Stream}(T_d, n, d, s, c, r, T_u, x)\\), return
   \\(\textrm{Fields}()\\).

 - If \\(T_{in} = \textrm{Bits}(b)\\), return
   \\(\textrm{Fields}(\varnothing : b)\\).

 - If \\(T_{in} = \textrm{Group}(N_{g,1} : T_{g,1}, N_{g,2} : T_{g,2}, ..., N_{g,n} : T_{g,n})\\),
   apply the following algorithm.

    - Initialize \\(N_o\\) and \\(b_o\\) to empty lists.
    - For all \\(i \in 1..n\\):
       - Unpack \\(\mathrm{fields}(T_{g,i})\\) into
         \\(\textrm{Fields}(N_{e,1} : b_{e,1}, N_{e,2} : b_{e,2}, ..., N_{e,m} : b_{e,m})\\).
       - For all \\(j \in 1..m\\):
          - If \\(N_{e,j} = \varnothing\\), append \\(N_{g,i}\\) to the end
            of \\(N_o\\). Otherwise, append the concatenation of \\(N_{g,i}\\)
            and \\(N_{e,j}\\), separated by a double underscore, to the end of
            \\(N_o\\).
          - Append \\(b_{e,j}\\) to the end of \\(b_o\\).
    - Return \\(\textrm{Fields}(N_{o,1} : b_{o,1}, N_{o,2} : b_{o,2}, ..., N_{o,p} : b_{o,p})\\),
      where \\(p = |N_o| = |b_o|\\).

 - If \\(T_{in} = \textrm{Union}(N_{u,1} : T_{u,1}, N_{u,2} : T_{u,2}, ..., N_{u,n} : T_{u,n})\\),
   apply the following algorithm.

    - Initialize \\(N_o\\) and \\(b_o\\) to empty lists.
    - If \\(n > 1\\):
       - Append `"tag"` to the end of \\(N_o\\).
       - Append \\(\left\lceil\log_2 n\right\rceil\\) to the end of \\(b_o\\).
    - \\(b_d := 0\\)
    - For all \\(i \in 1..n\\):
       - Unpack \\(\mathrm{fields}(T_{g,i})\\) into
         \\(\textrm{Fields}(N_{e,1} : b_{e,1}, N_{e,2} : b_{e,2}, ..., N_{e,m} : b_{e,m})\\).
       - \\(b_d := \max\left(b_d, \sum_{j=1}^m b_{e,j}\right)\\)
    - If \\(b_d > 0\\):
       - Append `"union"` to the end of \\(N_o\\).
       - Append \\(b_d\\) to the end of \\(b_o\\).
    - Return \\(\textrm{Fields}(N_{o,1} : b_{o,1}, N_{o,2} : b_{o,2}, ..., N_{o,p} : b_{o,p})\\),
      where \\(p = |N_o| = |b_o|\\).


# VERY MUCH TODO BELOW THIS POINT

Signals
-------

Given a logical stream type and a logical stream name (defined below), a list
of signals/ports can be constructed.

### Logical stream name

The logical stream name is a string consisting of letters, numbers, and/or
underscores, used to identify the logical stream.

The name cannot start or end with an underscore.

The name cannot start with a digit.

The name cannot be empty.

### Signal list construction

This section describes algorithmically how to convert from a logical stream
type and name to a list of \\(\textrm{Signal}\\)s (as defined below).

n ordered set of signals, defined as
\\(\textrm{Signal}(N : b, r, d)\\), where:

 - a name, guaranteed to:
    - consist of only letters, numbers, and underscores,
    - not start or end with an underscore,
    - not start with a digit,
    - not be empty, and
    - be case-insensitively unique;
 - a direction, either \\(\textrm{Forward}\\) or \\(\textrm{Reverse}\\);
 - a bit count, being either a positive integer, or \\(\varnothing\\) to
   indicate a scalar signal;
 - a default value, to be used when the opposite end of the interface (having a
   \\(\textrm{compatible}\\) logical stream type) does not define the signal.

#### Flattened logical stream type

Before describing the conversion from a logical stream type and name to its
corresponding signal list, we first define an intermediate representation,
referred to as the *flattened* logical stream type. This consists of:

 - an emptyable, ordered set of named asynchronous signals, where each signal
   has a positive bit count, and
 - an emptyable, ordered set of named, direction-annotated physical streams, as
   defined in the section on physical streams.

The signal/stream names cannot start or end with an underscore.

The signal/stream names cannot start with a digit.

The signal/stream names cannot be empty unless there is only one signal/stream.

The signal/stream names must be case-insensitively unique.

#### Conversion step one


