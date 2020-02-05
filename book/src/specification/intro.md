Introduction
============

This section describes the motivation and scope of the Tydi specification.

Background and motivation
-------------------------

As FPGAs become faster and larger, they are increasingly used within a
data center context to accelerate computational workloads. FPGAs can already
achieve greater compute density and power efficiency than CPUs and GPUs for
certain types of workloads, particularly those that rely on high streaming
throughput and branch-heavy computation. Examples of this are decompression,
SQL query acceleration, and pattern matching. The major disadvantage of FPGAs
is that they are more difficult to program than CPUs and GPUs, and that
algorithms expressed imperatively in the CPU/GPU domain often need to be
redesigned from the ground up to achieve competitive performance. Both the
advantages and disadvantages are due to the spatial nature of an FPGA: instead
of programming a number of processors, an FPGA designer "programs" millions of
basic computational elements not much more complex than a single logic gate
that all work parallel to each other, and describes how these elements are
interconnected. This extreme programmability comes at a cost of roughly an
order of magnitude in area and an order of magnitude in performance compared
to the custom silicon used to make CPUs and GPUs. Therefore, while imperative
algorithms can indeed be mapped to FPGAs more or less directly through
high-level synthesis (HLS) techniques or the use of softcores, typically an
algorithm needs at least two orders of magnitude of acceleration through clever
use of the spatial nature of the FPGA to be competitive.

Unfortunately, the industry-standard toolchains needed to program FPGAs only
take VHDL, Verilog, SystemC, (more recently through HLS) a subset of C++,
and/or visually designed data flow graphs as their input. The first three
provide few abstractions above the level of a single wire: while they do allow
the use of data types such as integers and structures to represent bundles of
wires, all control of what the voltages on those bundles of wires represent at
a certain point in time (if anything) remains up to the programmer. The latter
two techniques raise the bar slightly by presenting the designer with streams
and memory. However, they are vendor-specific, often require additional
licensing fees over the more traditional design entry methods, and in some
cases are even specific to a certain version of the vendor tool and/or a
certain FPGA device family.

This situation has given rise to a number of open-source projects that take
higher-level languages and transform them to vendor-agnostic VHDL or Verilog.
Examples of this are Chisel/FIRRTL and Clash, using generative Scala code and a
Haskell-like functional programming language as their respective inputs. Both
tools come with their own standard libraries of hardware components that can be
used to compose accelerators out of smaller primitives, similar to the data
flow design method described earlier, but with textual input and the advantage
of being vendor and device agnostic.

With the advent of these data flow composition tools, it is increasingly
important to have a common interface standard to allow the primitive blocks to
connect to each other. The de-facto standard for this has become the AMBA AXI4
interface, designed by ARM for their microcontrollers and processors. Roughly
speaking, AXI4 specifies an interface for device-to-memory connections (AXI4
and AXI4-lite), and a streaming interface (AXI4-stream) for intra-device
connections.

While AXI4 and AXI4-lite are of primary importance to processors due to their
memory-oriented nature, AXI4-stream is much more important for FPGAs due to
their spatial nature. However, because AXI4-stream is not originally designed
for FPGAs, parts of the specifications are awkward for this purpose. For
instance, AXI4-stream is byte oriented: it requires its data signal to be
divided into one or more byte lanes, and specifies (optional) control signals
that indicate the significance of each lane. Since FPGA designs are not at all
restricted to operating on byte elements, this part of the specification is
often ignored, and as such, any stream with a `valid`, `ready`, and one or more
`data` signals of any width has become the de-facto streaming standard. This is
reflected for instance by Chisel's built-in `Decoupled` interface type.

Within a single design this is of course not an issue — as long as both the
stream source and sink blocks agree on the same noncompliant interface, the
design will work. However, bearing in mind that there is an increasing number
of independently developed data flow oriented tools, each with their own
standard library, interoperability becomes an issue: whenever a designer needs
to use components from different vendors, they must first ensure that the
interfaces match, and if not, insert the requisite glue logic in between.

A similar issue exists in the software domain, where different programming
languages use different runtime environments and calling conventions. For
instance, efficiently connecting a component written in Java to a component
written in Python requires considerable effort. The keyword here is
"efficiently:" because Java and Python have very different ways of representing
abstract data in memory, one fundamentally has to convert from one
representation to another for any communication between the two components.
This serialization and deserialization overhead can and often does cost more
CPU time than the execution of the algorithms themselves.

The Apache Arrow project attempts to solve this problem by standardizing a way
to represent this abstract data in memory, and providing libraries for popular
programming languages to interact with this data format. The goal is to make
transferring data between two processes as efficient as sharing a pool of
Arrow-formatted memory between them. Arrow also specifies efficient ways of
serializing Arrow data structures for (temporary) storage in files or streaming
structures over a network, and can also be used by GPUs through CUDA. However,
FPGA-based acceleration is at the time of writing missing from the core
project. The Fletcher project attempts to bridge this gap, by providing an
interface layer between the Arrow in-memory format and FPGA accelerators,
presenting the memory to the accelerator in an abstract, tabular form.

In order to represent the complex, nested data types supported by Arrow, the
Fletcher project had to devise its own data streaming format on top of the
de-facto subset of AXI4-stream. Originally, this format was simply a means
to an end, and therefore, not much thought was put into it. Particularly, as
only Arrow-to-device interfaces (and back) were needed for the project, an
interface designed specifically for intra-device streaming is lacking; in
fact, depending on configuration, the reader and writer interfaces do not even
match completely. Clearly, a standard for streaming complex data types between
components is needed, both within the context of the Fletcher project, and
outside of it.

As far as the writers are aware, no such standard exists as of yet. Defining
such a standard in an open, royalty-free way is the primary purpose of this
document.

Goals
-----

 - Defining a streaming format for complex data types in the context of FPGAs
   and, potentially, ASICs, where "complex data types" include:

    * multi-dimensional sequences of undefined length;
    * unions (a.k.a. variants);
    * structures such as tuples or records.

 - Doing the above in as broad of a way as possible, without imposing
   unnecessary burdens on the development of simple components.

 - Allowing for minimization of area and complexity through well-defined
   contracts between source and sink on top of the signal specification itself.

 - Extensibility: this specification should be as usable as possible, even to
   those with use cases not foreseen by this specification.

Non-goals
---------

 - In this specification, a "streaming format" refers to the way in which the
   voltages on a bundle of wires are used to communicate data. We expressly do
   NOT mean streaming over existing communication formats such as Ethernet, and
   certainly not over abstract streams such as POSIX pipes or other
   inter-process communication paradigms. If you're looking for the former,
   have a look at Arrow Flight. The latter is part of Apache Arrow itself
   through their IPC format specification.

 - We do not intend to compete with the AXI4(-stream) specification.
   AXI4-stream is designed for streaming unstructured byte-oriented data;
   Tydi streams are for streaming structured, complex data types.

 - Tydi streams have no notion of multi-endpoint network-on-chip-like
   structures. Adding source and destination addressing or other routing
   information can be done through the `user` signal.

 - The primitive data type in Tydi is a group of bits. We leave the mapping
   from these bits to higher-order types such as numbers, characters, or
   timestamps to existing specifications.
