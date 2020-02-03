# Tydi: an open specification for complex data structures over hardware streams

# 1. Introduction

Tydi is an open-source project that aims to standardize interfaces for 
hardware components used in modern streaming dataflow designs.

The project consists of two parts: a set of *specifications* and a set of 
reference implementations of useful tools.

## Specification

Tydi contains a set of three specifications for the following, related 
concepts:

1. [Physical streams](./specification/physical.md)
2. [Logical streams](./specification/logical.md) (under construction)
3. [Streamlets](./specification/streamlet.md) (under construction)

**Physical streams** are hardware streams with their own valid/ready 
handshaking interface, transporting elementary data. Their exact bit-level 
representation and transfer behavior is defined through five parameters. 
These parameters are derived from logical streams.

**Logical streams** are one or multiple physical streams of some type from 
the Tydi type system. Types expressed in this type system determine which 
physical streams with which parameters make up the logical stream.

**Streamlets** are components that have Tydi logical streams as input and 
output.

The Tydi type system for logical streams allows the expression of data types 
that represent *dynamically-sized, nested and multidimensional data 
structures* in hardware.

## Tools

Using the Tydi specifications, reference implementations of tools are implemented and open-sourced.

1. [Generators](./tools/generators.md)
2. [Compositors](./tools/compositors.md)

**Generators** create HDL templates, that users may fill in with their 
desired behavior. Tydi aims to impose *no restrictions* on what 
hardware-description languages are used to work with the specification.
Tydi aims to provide generators for old HDLs, such as VHDL and Verilog, 
but also modern HDL

**Compositors** are tools that combine streamlets into larger designs.

## Example

*Warning: intended functionality shown, not necessary readily available*

If we want a reusable component that splits (non-extended) ASCII strings by 
whitespace characters, we can write a Streamlet Definition File: 
`StringSplit.sdf`:

```plaintext
SplitString             # Streamlet name

# Inputs:
x: Dim<Bits<7>>         # x is a logical stream of type Dim<Bits<8>>.
                        # Dim increases the dimensionality of the inner type.
                        # This means this input consists of a stream that
                        # transports a variable-length sequence of 8 bit 
                        # elements.
# Outputs:
y: Dim<Dim<Bits<7>>>    # This type allows the stream y to transport a
                        # sequence of sequences of 8 bit elements.
```

We can then generate a template in our HDL of preference, for example:

```console
$ tydi generate vhdl StringSplit.sdf
```

We obtain a VHDL package with a component declaration 
`StringSplit_pkg.gen.vhdl`, that adheres to the Tydi specification.

```vhdl
package StringSplit_pkg is

component StringSplit
    port(
        x_valid : in  std_logic;
        x_ready : out std_logic;
        x_data  : in  std_logic_vector(6 downto 0);
        x_strb  : in  std_logic_vector(0 downto 0);
        x_last  : in  std_logic_vector(0 downto 0);

        y_valid : out std_logic;
        y_ready : in  std_logic;
        y_data  : out std_logic_vector(6 downto 0);
        y_strb  : out std_logic_vector(0 downto 0);
        y_last  : out std_logic_vector(1 downto 0)
    );
end component;

end StringSplit_pkg;
```

An example of how to use this Streamlet in a larger, mixed-language design
will follow soon.
