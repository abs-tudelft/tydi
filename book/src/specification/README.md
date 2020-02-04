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
 - [Physical stream specification](physical.md): defines physical streams,
   carrying a stream of data and dimensionality information.
 - [Logical stream specification](logical.md): defines the building blocks
   for transferring complex data types by means of one or more physical
   streams.

The two specification sections are written such that the first paragraph of
each section *defines* a new concept, and the subsequent paragraphs *constrain*
it.

> Additional information, such as examples, a more intuitive description, or
> motivation, is written in these kinds of blocks.
