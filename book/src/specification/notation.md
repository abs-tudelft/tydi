Notation
========

The Tydi specification uses a flavor of mathematical notation to formally
represent the defined concepts and operations thereupon. The nonstandard part
of this notation is established in this section.

Nodes
-----

The structures needed by the specification are complex to the point that
simply using tuples becomes overly confusing. Therefore, we introduce a form
of named tuple, which we will call a node (the term "type" would be more
appropriate, but would be confusing within the context of this specification).

The notation for a node is as follows:

\\[\textrm{NodeName}(...)\\]

where \\(\textrm{NodeName}\\) is a title case identifier unique within the
specification, and \\(...\\) is a number of field names in some pattern. For
example:

\\[\textrm{Fields}(N_1 : b_1, N_2 : b_2, ..., N_n : b_n)\\]

A node carries with it implicit requirements on what the fields may be.
Whenever an operation constructs a node, the operation is written in a way
that these requirements are guaranteed by the preconditions of the operation.

The field names of nodes are significant. When an operation depends on one of
the fields of the node, the fields may be "accessed" by subscripting the field
name. For instance, if \\(F\\) is a \\(\textrm{Fields}\\) node, \\(F_n\\)
represents the value for \\(n\\), and \\(F_N\\) represents the tuple consisting
of \\(N_1, N_2, ..., N_n\\) of the node.

When the field being indexed is clear from context, the field itself is usually
omitted; for instance, using just \\(N\\) implies the same thing as \\(F_N\\)
if there is only one \\(\textrm{Fields}\\) node being discussed, and nothing
else is using \\(N\\) for anything.

Nodes may also be unpacked into differently named values using the following
notation:

> Unpack \\(F'\\) into \\(\textrm{Fields}(N'_1 : b'_1, N'_2 : b'_2, ..., N'_n : b'_n)\\).

The following discussion may then use for instance \\(N'\\) to refer to
\\(F'_N\\).

In some cases, only some of the fields of a node need to have a certain value
for the following discussion to apply. We will then write only those fields
within the parentheses, prefixing the values with the name and an equals sign,
followed by an ellipsis to signify the unconstrained fields. For example,
\\(\textrm{Fields}(n=2, ...)\\) signifies a \\(\textrm{Fields}\\) node with
exactly two \\(N\\) and two \\(b\\), but the values thereof are unconstrained.

Functions
---------

Due to the highly generic nature of the specification, parts of it are
described algorithmically. The algorithms are referred to as functions. Like
functions in a typical programming language, they have a signature, written
as follows:

\\[\textrm{functionName}(...) \rightarrow \textrm{ReturnType}\\]

where \\(\textrm{functionName}\\) is a camelcase identifier unique within the
specification, \\(...\\) is a number of argument names in some pattern, and
\\(\textrm{ReturnType}\\) specifies what is returned, usually in the form of a
node name.

A function carries with it implicit requirements on what its parameters may be.
Whenever an operation uses a function, the operation is written in a way
that these requirements are guaranteed by the preconditions of the operation.

