library ieee;
use ieee.std_logic_1164.all;

package test_library is

component sink_com
  port(
    clk : in std_logic;
    rst : in std_logic;
    in_sink_valid : in std_logic;
    in_sink_ready : out std_logic;
    in_sink_data : in std_logic_vector(263 downto 0);
    in_sink_stai : in std_logic_vector(2 downto 0);
    in_sink_endi : in std_logic_vector(2 downto 0);
    in_sink_strb : in std_logic_vector(7 downto 0)
  );
end component;

--- Verify that multiple enums can have the same names and this construct still works:
type pass_in_pass_data_tag_type is (a, c);
type pass_in_pass_data_type_length_array_type is array(pass_in_pass_data_tag_type) of integer;
constant pass_in_pass_data_type_index : pass_in_pass_data_type_length_array_type := (
  a => 0,
  c => 1
);
constant pass_in_pass_data_type_length : pass_in_pass_data_type_length_array_type := (
  a => 32,
  c => 8
);

subtype sink_in_sink_data_tag_type is std_logic_vector(0 downto 0);
type sink_in_sink_data_tag_enum is (a, b);

type sink_in_sink_data_tag_encoding_array_type is array(sink_in_sink_data_tag_enum) of sink_in_sink_data_tag_type;
constant sink_in_sink_data_tag_encoding : sink_in_sink_data_tag_encoding_array_type := (
  a => "0",
  b => "1"
);

type sink_in_sink_data_type_length_array_type is array(sink_in_sink_data_tag_enum) of natural range<>;
constant sink_in_sink_data_type_length : sink_in_sink_data_type_length_array_type := (
  a => 32,
  b => 8
);

type sink_in_sink_data_type is record
  tag : sink_in_sink_data_tag_type;
  a : std_logic_vector(31 downto 0);
  b : std_logic_vector(7 downto 0);
end record;

type sink_in_sink_data_array_type is array (0 to 7) of sink_in_sink_data_type;

type sink_in_sink_dn_type is record
  valid : std_logic;
  data : sink_in_sink_data_array_type;
  stai : std_logic_vector(2 downto 0);
  endi : std_logic_vector(2 downto 0);
  strb : std_logic_vector(7 downto 0);
end record;

type sink_in_sink_up_type is record
  ready : std_logic;
end record;

component sink
  port(
    clk : in std_logic;
    rst : in std_logic;
    in_sink_dn : in sink_in_sink_dn_type;
    in_sink_up : out sink_in_sink_up_type
  );
end component;

end test_library;