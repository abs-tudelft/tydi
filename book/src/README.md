![Tydi logo](./tydi_logo.svg)

# Tydi: an open specification for complex data structures over hardware streams

# 1. Introduction

Tydi is an open-source project that aims to standardize interfaces for
hardware components used in modern streaming dataflow designs.

The project consists of two parts: the Tydi specification and a set of
reference implementations of useful tools.

## Specification

The Tydi (Typed dataflow interface) specification is at the core of the
project: it defines what an interface should look like and constrains its
behavior, based on a constructively defined, low-level data type. Start
reading [here](./specification/index.md).

## Tools

Using the Tydi specifications, reference implementations of tools are
implemented and open-sourced.

1. [Generators](./tools/generators.md) (under construction)
2. [Compositors](./tools/compositors.md) (under construction)

**Generators** create HDL templates, that users may fill in with their
desired behavior. Tydi aims to impose *no restrictions* on what
hardware-description languages are used to work with the specification.
Tydi aims to provide generators for old HDLs, such as VHDL and Verilog,
but also modern HDL

**Compositors** are tools that combine streamlets into larger designs.
