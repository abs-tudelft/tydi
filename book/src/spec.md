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

| \\(C\\) | Description                                                                         |
|---------|-------------------------------------------------------------------------------------|
| < 8     | Only one sequence can be terminated per transfer.                                   |
| < 7     | Which data lanes are enabled can be described with a simple range.                  |
| < 6     | The range of enabled data lanes must always start with lane zero.                   |
| < 5     | All lanes must be enabled for all but the last transfer of the innermost sequence.  |
| < 4     | The `last` flag cannot be postponed until after the transfer of the last element.   |
| < 3     | Innermost sequences must be transferred in consecutive cycles.                      |
| < 2     | Whole outermost instances must be transferred in consecutive cycles.                |

A stream is considered to be normalized when \\(C < 4\\). At this complexity
level, there is a one-to-one mapping between the transferred data and the
actual transfers for any given \\(N\\). This is called the canonical
representation of the data for a given \\(N\\).

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

Signals
-------

A physical stream is comprised of the following signals.

| Name    | Origin | Purpose                                                                                |
|---------|--------|----------------------------------------------------------------------------------------|
| `valid` | Source | Stalling the data stream due to the source not being ready.                            |
| `ready` | Sink   | Stalling the data stream due to the sink not being ready.                              |
| `data`  | Source | Data transfer of \\(N\\) \\(E\\)-bit elements.                                         |
| `last`  | Source | Indicating the last transfer for \\(D\\) levels of nested packets.                     |
| `stai`  | Source | Start index; encodes the index of the first valid lane.                                |
| `endi`  | Source | End index; encodes the index of the last valid lane.                                   |
| `strb`  | Source | Strobe; encodes individual lane validity for \\(C \ge 8\\), empty sequences otherwise. |
| `user`  | Source | Additional control information carried along with the stream.                          |

The `valid` and `ready` signals are scalars, while all the other signals are
bit vectors with the following widths.

| Name    | Width                         |
|---------|-------------------------------|
| `valid` | *scalar*                      |
| `ready` | *scalar*                      |
| `data`  | \\(N \times E\\)              |
| `last`  | \\(N \times D\\)              |
| `stai`  | \\(\lceil \log_2{N} \rceil\\) |
| `endi`  | \\(\lceil \log_2{N} \rceil\\) |
| `strb`  | \\(N\\)                       |
| `user`  | \\(U\\)                       |

### Clock

All signals are synchronous to a single clock signal, shared between the source
and sink. It is up to the user to manage clock domains and clock domain
crossing where needed, and specify the edge sensitivity of the clock(s).

### Reset

The source and sink need not share the same reset signal, because the
requirements on the `valid` and `ready` signals ensure that no transfers occur
when either the source or the sink is held under reset. It is up to the user to
manage the synchronicity and sensitivity of the reset signal(s).

### Detailed signal description

#### `valid` signal description

> The `valid` signal has the same semantics as the `TVALID` AXI4-stream signal.
> These semantics are repeated here.

The `valid` signal indicates the presence of a valid transfer payload of the
other downstream signals. It is active-high.

The state of the `valid` signal is always significant.

> That is, if the source is in some undefined or "uninitialized" state, `valid`
> *must* nonetheless be low.

The downstream signals, including `valid`, must be stable (maintain their
state) from the first active clock edge in which `valid` is asserted until
acknowledgement of the transfer. Note that the validating and acknowledging
clock edge can be one and the same if `ready` is already asserted during the
first clock cycle; in this case there are no stability requirements.

> In other words, a transfer may not be cancelled or modified after validation.
> This allows a sink to start doing some sort of multicycle processing as soon
> as `valid` is asserted, only asserting `ready` when it is done, without
> having to worry about cancellation or changes in the transfer content. This
> prevents the need for extra holding registers or latches otherwise needed to
> keep the original transfer.

When a transfer is acknowledged, either the next transfer is presented on the
downstream signals and `valid` remains asserted, or `valid` is released.

*[\\(C < 3\\)]* `valid` may only be released when lane \\(N - 1\\) of the
`last` signal in the acknowledged transfer is nonzero.

*[\\(C < 2\\)]* `valid` may only be released when lane \\(N - 1\\) of the
`last` signal in the acknowledged transfer is all ones.

`valid` must be released while the source is being reset.

> This prevents spurious transfers if the sink has a different reset source.

#### `ready` signal description

> The `ready` signal has the same semantics as the `TREADY` AXI4-stream signal.
> These semantics are repeated here.

The `ready` signal indicates acknowledgement of a transfer initiated through
`valid`. It is active-high.

The state of the `ready` signal is significant only while `valid` is asserted.

> That is, a source must never wait for `ready` to assert before it asserts
> `valid`. Doing so can lead to deadlock. However, a sink *may* wait for
> `valid` to assert before asserting `ready`.

A transfer is considered "handshaked" when both `valid` and `ready` are
asserted during the active clock edge of the clock domain common to the source
and the sink.

`ready` must be released while the sink is being reset.

> This prevents transfers from being lost if the source has a different reset
> source.

#### `data` signal description

The `data` signal consists of `N` concatenated lanes, each carrying a number of
data element fields, totaling to `E` bits per lane.

The state of lane \\(i\\) in the `data` signal is significant only while
`valid` is asserted and data lane \\(i\\) is *active*. Data lane \\(i\\) is
considered active if and only if

 - bit \\(i\\) of `strb` is asserted,
 - the unsigned integer interpretation of `endi` is greater than or equal to
   \\(i\\), *and*
 - the unsigned integer interpretation of `stai` is less than or equal to
   \\(i\\).

> The redundancy of the `endi` and `stai` signals given the presence of the
> `strb` signal (only applicable for \\(C \ge 7\\)) has to do with the rule
> that it must always be possible to connect a source with low complexity to a
> sink with high complexity. That is, while a source that desires individual
> control over the lanes and thus has \\(C \ge 7\\)) would probably always
> drive `endi` to \\(N-1\\) and `stai` to 0, a sink with complexity
> \\(C \ge 7\\)) still needs to accept input from sources that do use `endi`
> and `stai`.
>
> The overhead incurred by this should be negligible in practice. First of all,
> because \\(C \ge 7\\)) with significant \\(N\\) will be quite large to begin
> with, so managing two additional signals of size
> \\(\lceil \log_2{N} \rceil\\) probably doesn't matter much. Secondly, in most
> cases, constant propagation will remove the unneeded `endi` and `stai`
> signals whenever both source and sink have \\(C \ge 7\\). Finally, even if
> the above fails, going from `stai` + `endi` + `strb` to the internal lane
> enable signals in a sink is trivial for modern FPGAs and realistic \\(N\\),
> as the less-than-equal and greater-than-equal comparisons fit in a single
> 6-LUT up to and including \\(N = 64\\), resulting in just three decoding
> LUTs per lane (one for `stai`, one for `endi`, and one 3-input AND gate to
> merge the outputs of the first two and the `strb` bit).

The significant lanes are interpreted ordered by increasing index for each
transfer.

#### `last` signal description

The `last` signal consists of `N` concatenated lanes, each carrying a
termination bit for each nested sequence represented by the physical stream.
The `last` bit for lane \\(i\\) and dimension \\(j\\) (i.e., bit
\\(i \cdot D + j\\)) being asserted indicates completion of a sequence with
nesting level \\(D - j - 1\\), containing all elements streamed between the
previous lane/transfer this occured for \\(j' \ge j\\) (or system reset, if
never) exclusively and this point inclusively, where nesting level is defined
to be 0 for the outermost list, 1 for its inner list, up to \\(D - 1\\) for
the innermost list. It is active-high.

> For example, one way to represent the value
> `["Hello", "World"], ["Tydi", "is", "nice"], [""], []` is as follows:
> ```text
> reset released here
>         |                      transfer -->
>         v      A              B              C              D
>
> last    "000100000000" "000011000000" "000001000100" "000010010000"
> data       "WolleH"       "yTdlro"       "insiid"       "----ec"
> strb       "111111"       "111111"       "111111"       "000011"
>
>         .------------. .------------. .------------. .------------.
>       0 | data:  'H' | | data:  'o' | | data:  'd' | | data:  'c' |
>         | last:  0 0 | | last:  0 0 | | last:  0 0 | | last:  0 0 |
>         |------------| |------------| |------------| |------------|
>       1 | data:  'e' | | data:  'r' | | data:  'i' | | data:  'e' |
>         | last:  0 0 | | last:  0 0 | | last:  0 1 | | last:  0 0 |
>         |------------| |------------| |------------| |------------|
> lane  2 | data:  'l' | | data:  'l' | | data:  'i' | | data:   -  |
> index   | last:  0 0 | | last:  0 0 | | last:  0 0 | | last:  0 1 |
>   |     |------------| |------------| |------------| |------------|
>   v   3 | data:  'l' | | data:  'd' | | data:  's' | | data:   -  |
>         | last:  0 0 | | last:  1 1 | | last:  0 1 | | last:  1 0 |
>         |------------| |------------| |------------| |------------|
>       4 | data:  'o' | | data:  'T' | | data:  'n' | | data:   -  |
>         | last:  0 1 | | last:  0 0 | | last:  0 0 | | last:  1 1 |
>         |------------| |------------| |------------| |------------|
>       5 | data:  'W' | | data:  'y' | | data:  'i' | | data:   -  |
>         | last:  0 0 | | last:  0 0 | | last:  0 0 | | last:  1 0 |
>         '------------' '------------' '------------' '------------'
> ```
> `"Hello"` is delimited by the reset condition and the asserted `last` bit
> in transfer A, lane 4, for \\(j = 0\\) (innermost). `"World"` is delimited
> by the aforementioned and the `last` bit in transfer B, lane 3, also for
> \\(j = 0\\). The `last` bit for B3/\\(j = 1\\) is also asserted, which
> delimits `["Hello", "World"]` along with the reset condition; both the
> `"Hello"` and `"World"` sequence are fully contained by the delimited
> portion of the stream, so the outer list has those as its two entries.
> `"Tydi"` is delimited by B3 and C1, `"is"` by C1 and C3, and `"nice"` by
> C3 and D2 for \\(j = 0\\). Note that the D2 is postponed beyond the last
> element (D1) by means of deactivating data lane D2. The `last` flag for
> the surrounding list is similarly postponed to D3; note though that data
> lane D3 *must* be inactive, *or* the last bit for D3/\\(j = 0\\) must be
> asserted for the data stream to be legal. The next `""` is delimited by
> D3 (\\(j' = 1\\)) and D4 (\\(j = 0\\)), and `[""]` is delimited by D3 and D4
> for \\(j = 1\\). The final `[]` is delimited by D4 and D5 for \\(j = 1\\).
> It does not contain an inner list item because D5/\\(j = 0\\) is not
> asserted, so the outer list is empty.

The state of lane \\(i\\) in the `data` signal is significant only while
`valid` is asserted.

> Note that the validity of the `last` lanes is thus *not* controlled by
> which data lanes are active. This allows sequences to be terminated without
> sending an element; this is in fact required in order to send empty
> sequences, but also allows postponing the end-of-sequence marker until after
> the last element (if complexity allows) as the previous example shows.

It is illegal to assert `last` bit \\(j\\) without first terminating all
\\(j' < j\\), if any data lanes were active since the previous assertion of
`last` bit \\(j" = 0\\).

> For example, the following is illegal because of the above:
>
> ```text
>         .------------.
>       0 | data:  '1' |
>         | last:  0 0 |
>         |------------|
>       1 | data:  '2' |
>         | last:  0 1 |
>         |------------|
> lane  2 | data:  '3' |
> index   | last:  0 0 |
>   |     |------------|
>   v   3 | data:  '4' |
>         | last:  1 0 |
>         |------------|
>       4 | data:  '5' |
>         | last:  0 0 |
>         |------------|
>       5 | data:  '6' |
>         | last:  1 1 |
>         '------------'
> ```
>
> An argument could be made that this encodes `[[1, 2], 3, 4], [[5, 6]]`, but
> allowing this interpretation would make the stream significantly less
> type-safe, and there would be no way to encode non-list elements before the
> last inner list in the outer list without introducing a `start` signal as
> well.
>
> An argument could also be made that this encodes `[[1, 2]], [[3, 4, 5, 6]]`,
> because the `last` flag delimiting `[1, 2]` falls within the first outer list
> and the `last` flag delimiting `[3, 4, 5, 6]` falls within the second. However,
> This would unnecessarily complicate sink logic, make manual interpretation of
> the data structure more context-sensitive, and so on.
>
> One could also argue that it encodes `[[1, 2]], [[5, 6]]`, because `3` and
> `4` are not enclosed by any inner list, but then it makes more sense to just
> deactivate the lanes.
>
> Ultimately, this representation is ambiguous or at least leads to confusion,
> and is therefore illegal.

*[\\(C < 8\\)]* All last bits for lanes 0 to \\(N - 2\\) inclusive must be
driven low by the source, and may be ignored by the sink.

> The above rule ultimately means that the `last` information is transferred on
> a transfer basis instead of on element basis, similar to AXI4-stream. This
> can significantly decrease decoding complexity, but only allows one innermost
> sequence to be transferred per cycle regardless of \((N\\) and the length of
> the sequence.

*[\\(C < 4\\)]* It is illegal to assert `last` bit \\(j\\) without also
terminating all \\(j' < j\\) within the same lane.

*[\\(C < 4\\)]* It is illegal to assert the `last` bit for \\(j = 0\\) when
the respective data lane is inactive, except for empty sequences.

> The above two rules mean that the `last` flags cannot be postponed.

#### `stai` signal description

The `stai` signal (start index) is used to deactivate data lanes with an index
lower than the encoded binary number. It is a bit vector of size
\\(\lceil \log_2{N} \rceil\\) to be able to encode the full index range.

The state of the `stai` signal is significant only while `valid` is asserted.

It is illegal to drive `stai` to the value \\(N\\) or greater.

> This implies that `stai` cannot be used to disable the last lane.

*[\\(C < 6\\)]* `stai` must always be driven to 0 by the source, and may be
ignored by the sink.

#### `endi` signal description

The `endi` signal (end index) is used to deactivate data lanes with an index
greater than the encoded binary number. It is a bit vector of size
\\(\lceil \log_2{N} \rceil\\) to be able to encode the full index range.

> Note that `endi` cannot be used to disable the first lane.

The state of the `endi` signal is significant only while `valid` is asserted.

It is illegal to drive `endi` to the value \\(N\\) or greater.

> This would semantically not be different from driving \\(N - 1\\), so there
> is no reason to burden the sink logic by allowing this case.

It is illegal to drive `endi` to a value less than `stai`.

> This would essentially encode a VHDL null range, which, if allowed, could be
> used to encode empty lists. However, detecting this condition is relatively
> hard on FPGA resources (timing in particular), typically requiring a carry
> chain for \\(N > 8\\). Instead, use `strb` for this purpose; at lower
> complexities all `strb` bits must be equal, so only a single bit has to be
> checked to determine whether a transfer carries any data.

*[\\(C < 5\\)]* `endi` must be driven to \\(N - 1\\) by the source when `last`
is zero, and may be ignored by the sink in this case.

> Together with the other complexity rules up to this level, this means that
> all lanes must be used for all but the last transfer containing elements for
> the innermost list. This furthermore implies that the element with innermost
> index \\(i\\) is always transferred on data lane \\(i \mod N\\).

#### `strb` signal description

The `strb` signal (strobe) is used to deactivate individual data lanes. It is
a bit vector of size \\(N\\). If a `strb` bit is low, the associated data lane
is deactivated.

> Note that the opposite ("`strb` high -> activated) is *not* necessarily true
> due to `endi` and `stai`.

The state of the `strb` signal is significant only while `valid` is asserted.

*[\\(C < 8\\)]* All `strb` bits must be driven to the same value by the source.
The sink only needs to interpret one of the bits.

> This effectively reduces the `strb` signal to a single bit indicating whether
> the transfer is empty (low) or not (high), as it is illegal to deactivate all
> data lanes by driving `endi` < `stai`.

#### `user` signal description

The `user` signal is the concatenation of a number of user-specified fields,
carrying additional transfer-oriented information. The significance of this is
user-defined.

The state of the `user` signal is not significant when `valid` is not asserted.

> The opposite is not necessarily true; it is up to the user's specifications
> if (part of) the user signal can be insignificant despite `valid` being
asserted.

### Signal omission

Not all signals are always needed. The following table shows the conditions for
a signal to be relevant. If the condition is false, the signal may be omitted
on the interface. When two interfaces with differing but compatible
complexities are connected together, the default value specified in the table
below must be driven for any omitted signals.

| Name    | Condition                                   | Default   |
|---------|---------------------------------------------|-----------|
| `valid` | \\(\top\\)                                  | `'1'`     |
| `ready` | \\(\top\\)                                  | `'1'`     |
| `data`  | \\(E > 0\\)                                 | all `'0'` |
| `last`  | \\(D \ge 1\\)                               | all `'1'` |
| `stai`  | \\(C \ge 6 \wedge N > 1\\)                  | 0         |
| `endi`  | \\((C \ge 5 \vee D \ge 1) \wedge N > 1\\)   | \\(N-1\\) |
| `strb`  | \\(C \ge 7 \vee D \ge 1\\)                  | all `'1'` |
| `user`  | \\(U > 0\\)                                 | all `'0'` |

### Interface conventions

When there are name conflicts (for instance due to multiple physical streams
existing on the hierarchical level of an interface) the signal names must be
prefixed by a unique stream name, delimited using an underscore.

The canonical representation for the `data` and `user` signals is the LSB-first
concatenation of the contained fields. For \\(N > 1\\), the lanes are
concatenated LSB first, such that the lane index is major and the field index
is minor.

Where applicable, the signals must be listed in the following order: `dn`,
`up`, `valid`, `ready`, `data`, `last`, `stai`, `endi`, `strb`, `user`.

> This is mostly just a consistency thing, primarily because it helps to get
> used to a single ordering when interpreting simulation waveforms. It may also
> simplify tooling.

The signal names must be lowercase.

> This is for interoperability between languages that differ in case
> sensitivity.

#### Alternative representation

To improve code readability in hardware definition languages supporting array
and aggregate constructs (record, struct, ...), the following changes are
permissible. However, it is recommended to fall back to the conventions above
for interoperability with other streamlets on the "outer" interfaces of an IP
block.

 - `valid`, `data`, `last`, `stai`, `endi`, `strb`, and `user` may be bundled
   in an aggregate type named `<stream-name>_dn_type` with signal name
   `<stream-name>_dn`.

 - `ready` may be "bundled" in an aggregate type named `<stream-name>_up_type`
   with signal name `<stream-name>_up` for symmetry.

 - If the language allows for signal direction reversal within a bundle, all
   stream signals may also be bundled into a single type named
   `<stream-name>_type`, with `ready` in the reverse direction.

 - The data and user fields may be bundled in aggregate types named
   `<stream-name>_data_type` and `<stream-name>_user_type` respectively. The
   `data` signal becomes an array of `<stream-name>_data_type`s from 0 to
   \\(N - 1\\) if \\(N > 1\\) or this is otherwise desirable.

 - Data and user fields consisting of a single bit may be interpreted as either
   a bit vector of size one or a scalar bit depending on context.

 - Fields with a common underscore-delimited prefix may be aggregated
   recursively using `<stream-name>_data_<common-prefix>_type`, in such a
   way that the underscores in the canonical signal name are essentially
   replaced with the hierarchy separator of the hardware definition language.

#### Arrays, vectors, and concatenations

Where applicable, the bitrange for bit vector signals is always `n-1 downto 0`,
where `n` is the number of bits in the signal.

Concatenations of bit vector signals are done LSB-first, i.e., lower indexed
entries use lower bit indices and thus occur on the right-hand side when the
bit vector is written as a binary number.

> Note that this results in the inverse order when using the concatenation
> operator in VHDL (and possibly other languages). LSB-first order is chosen
> because it is more intuitive when indexing the vector. Ultimately this is
> just a convention.

Where applicable, the range for arrays is always `0 to n-1`, where `n` is the
number of entries in the array.

> This is the more natural order for array-like structures, actually putting
> the first entry on the left-hand side.
