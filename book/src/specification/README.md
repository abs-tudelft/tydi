Tydi specification
==================

The Tydi specification, short for Typed dataflow interface specification,
aims to provide a standardized way in which complex data types can be
transferred from one circuit to another, within the context of FPGAs or
ASICs.

How to read this document
-------------------------

The specification is comprised of the following sections.

 - [Introduction](intro.md): defines the motivation and scope of this
   specification.

 - [Notation](notation.md): defines the mathematical notation used by the
   remainder of the specification.

 - [Physical stream specification](physical.md). **Physical streams** are
   hardware streams with their own valid/ready handshaking interface,
   transporting elementary data and dimensionality information. Their exact
   bit-level representation and transfer behavior is defined through five
   parameters. These parameters are normally derived from logical streams.

 - [Logical stream specification](logical.md). **Logical streams** are bundles
   of one or multiple physical streams of some type from the Tydi type system.
   Types expressed in this type system determine which physical streams with
   which parameters make up the logical stream. This section also introduces
   **streamlets**, the Tydi name for components that have logical streams as
   inputs and/or outputs.

The two specification sections are written such that the first paragraph of
each section *defines* a new concept, and the subsequent paragraphs *constrain*
it.

> Additional information, such as examples, a more intuitive description, or
> motivation, is written in these kinds of blocks.
