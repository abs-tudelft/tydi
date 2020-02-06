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
defined as \\(\mathrm{Stream}(T_d, n, d, s, c, r, T_u)\\), where:

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
   > physical stream will transfer `[[2, 3], [5]], [[7]]`. If \mathrm{Flatten}
   > is used for the inner \\(\textrm{Stream}\\) instead, the data transferred
   > by the second physical stream reduces to `[2, 3], [5], [7]`; the structure
   > of the outer sequence is implied by the structure transferred by the first
   > physical stream.
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

 - \\(T_{source} = \mathrm{Stream}(T_d, n, d, s, c, r, T_u)\\),
   \\(T_{sink} = \mathrm{Stream}(T'_d, n, d, s, c', r, T_u)\\),
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

Split function
--------------

This section defines the function
\\(\mathrm{split}(T_{in}) \rightarrow \textrm{SplitStreams}(T_{signals}, N_1 : T_1, N_2 : T_2, ..., N_n : T_n)\\),
where \\(T\\) is any logical stream types, T_{signals} is a logical stream type
consisting of only element-manipulating nodes, all \\(T_{1..n}\\) are
\\(\textrm{Stream}\\) nodes with the data and user subtypes consisting
of only element-manipulating nodes, all \\(N\\) are case-insensitively
unique, emptyable strings consisting of letters, numbers, and/or underscores,
not starting or ending in an underscore, and not starting with a digit, and
\\(n\\) is a nonnegative integer.

> Intuitively, it splits a logical stream type up into simplified stream types,
> where \\(T_{signals}\\) contains only the information for the user-defined
> signals, and \\(T_i\\) contains only the information for the physical stream
> with index \\(i\\) and name \\(N_i\\).
>
> The names can be emptyable, because if there are no \\(Group\\)s or
> \\(Union\\)s, the one existing stream or signal will not have an intrinsic
> name given by the type. Note that the names here can include double
> underscores.

\\(\mathrm{split}(T_{in})\\) is evaluated as follows.

 - If \\(T_{in} = \mathrm{Stream}(T_d, n, d, s, c, r, T_u)\\), apply the
   following algorithm.

    - \\(T_{data} := \mathrm{split}(T_{in})\_{signals}\\) (i.e., the
      \\(T_{signals}\\) item of the tuple returned by the \\(\mathrm{split}\\)
      function)
    - \\(T_{main} := \mathrm{Stream}(T_{data}, n, d, s, c, r, T_u)\\)
    - Initialize \\(N\\) and \\(T\\) to empty lists.
       - Extend \\(N\\) with \\(\mathrm{split}(T_d)\_{names}\\) (i.e.,
         all stream names returned by the \\(\mathrm{split}\\) function).
       - For all \\(T' \in \mathrm{split}(T_d)\_{streams}\\) (i.e.,
         for all named streams returned by the \\(\mathrm{split}\\) function),
          - Unpack \\(T'\\) into \\(\mathrm{Stream}(T'\_d, n', d', s', c', r', T'\_u)\\).
          - If \\(r = \textrm{Reverse}\\), reverse the direction of \\(s'\\).
          - If \\(s = \textrm{Flatten}\\) or \\(s = \textrm{FlatDesync}\\),
            assign \\(s' := \textrm{FlatDesync}\\).
          - If \\(s' \ne \textrm{Flatten}\\) and \\(s \ne \textrm{FlatDesync}\\),
            assign \\(d' := d' + d\\).
          - \\(n' := n' \cdot n\\).
          - Append \\(\mathrm{Stream}(T'\_d, n', d', s', c', r', T'\_u)\\) to
            \\(T\\).
    - Return \\(\textrm{SplitStreams}(\textrm{Null}, \varnothing : T_{main} : T_1, N_2 : T_2, ..., N_m : T_m)\\).

 - If \\(T_{in} = \textrm{Null}\\) or \\(T_{in} = \textrm{Bits}(b)\\),
   return \\(\textrm{SplitStreams}(T_{in})\\).

 - If \\(T_{in} = \textrm{Group}(N_{g,1} : T_{g,1}, N_{g,2} : T_{g,2}, ..., N_{g,n} : T_{g,n})\\)
   or \\(T_{in} = \textrm{Union}(N_{g,1} : T_{g,1}, N_{g,2} : T_{g,2}, ..., N_{g,n} : T_{g,n})\\),
   apply the following algorithm.

    - \\(T_{signals} := \mathrm{split}(T_{in})\_{signals}\\) (i.e., the
      \\(T_{signals}\\) item of the tuple returned by the \\(\mathrm{split}\\)
      function)
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
    - Return \\(\textrm{SplitStreams}(T_{signals}, N_1 : T_1, N_2 : T_2, ..., N_m : T_m)\\).

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



