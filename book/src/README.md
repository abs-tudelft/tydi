# Tydi: an open specification for complex data structures over hardware streams

# 1. Introduction

Tydi is an open-source project that aims to standardize interfaces for hardware components used in modern streaming dataflow designs.

The project consists of two parts: a set of *specifications* and a set of reference implementations of useful tools.

## Specification

Tydi contains a set of three specifications for the following, related concepts:

1. [Physical streams](./specification/physical.md)
2. [Logical streams](./specification/logical.md) (under construction)
3. [Streamlets](./specification/streamlet.md) (under construction)

**Physical streams** are hardware streams with their own valid/ready handshaking interface, transporting elementary data. Their exact bit-level representation and transfer behavior is defined through five parameters. These parameters are derived from logical streams.

**Logical streams** are one or multiple physical streams of some type from the Tydi type system. Types expressed in this type system determine which physical streams with which parameters make up the logical stream.

**Streamlets** are components that have Tydi logical streams as input and output.

The Tydi type system for logical streams allows the expression of data types that represent *dynamically-sized, nested and multidimensional data structures* in hardware.

## Tools

Using the Tydi specifications, reference implementations of tools are implemented and open-sourced.

1. [Generators](./tools/generators.md)
2. [Compositors](./tools/compositors.md)

**Generators** create HDL templates, that users may fill in with their desired behavior. Tydi aims to impose *no restrictions* on what hardware-description languages are used to work with the specification.

**Compositors** are tools that combine streamlets into larger designs.
