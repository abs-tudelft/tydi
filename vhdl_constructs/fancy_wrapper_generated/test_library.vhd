library ieee;
use ieee.std_logic_1164.all;

package test_library is

component passthrough_stub_com
  port(
    clk : in std_logic;
    rst : in std_logic;
    in_pass_valid : in std_logic;
    in_pass_ready : out std_logic;
    in_pass_data : in std_logic_vector(263 downto 0);
    in_pass_stai : in std_logic_vector(2 downto 0);
    in_pass_endi : in std_logic_vector(2 downto 0);
    in_pass_strb : in std_logic_vector(7 downto 0);
    in_pass2_valid : in std_logic;
    in_pass2_ready : out std_logic;
    in_pass2_data : in std_logic_vector(117 downto 0);
    in_pass2_strb : in std_logic_vector(0 downto 0);
    out_pass_valid : out std_logic;
    out_pass_ready : in std_logic;
    out_pass_data : out std_logic_vector(263 downto 0);
    out_pass_stai : out std_logic_vector(2 downto 0);
    out_pass_endi : out std_logic_vector(2 downto 0);
    out_pass_strb : out std_logic_vector(7 downto 0)
  );
end component;

type passthrough_stub_in_pass_data_type is record
  -- Variants: a, b
  tag : std_logic_vector(0 downto 0);
  a : std_logic_vector(31 downto 0);
  b : std_logic_vector(7 downto 0);
end record;

type passthrough_stub_in_pass_data_array_type is array (0 to 7) of passthrough_stub_in_pass_data_type;

type passthrough_stub_in_pass_dn_type is record
  valid : std_logic;
  data : passthrough_stub_in_pass_data_array_type;
  stai : std_logic_vector(2 downto 0);
  endi : std_logic_vector(2 downto 0);
  strb : std_logic_vector(7 downto 0);
end record;

type passthrough_stub_in_pass_up_type is record
  ready : std_logic;
end record;

type passthrough_stub_in_pass2_data_dn_type is record
  op1 : std_logic_vector(63 downto 0);
  op2 : std_logic_vector(53 downto 0);
end record;

type passthrough_stub_in_pass2_dn_type is record
  valid : std_logic;
  data : passthrough_stub_in_pass2_data_dn_type;
  strb : std_logic_vector(0 downto 0);
end record;

type passthrough_stub_in_pass2_up_type is record
  ready : std_logic;
end record;

type passthrough_stub_out_pass_data_type is record
  -- Variants: a, b
  tag : std_logic_vector(0 downto 0);
  a : std_logic_vector(31 downto 0);
  b : std_logic_vector(7 downto 0);
end record;

type passthrough_stub_out_pass_data_array_type is array (0 to 7) of passthrough_stub_out_pass_data_type;

type passthrough_stub_out_pass_dn_type is record
  valid : std_logic;
  data : passthrough_stub_out_pass_data_array_type;
  stai : std_logic_vector(2 downto 0);
  endi : std_logic_vector(2 downto 0);
  strb : std_logic_vector(7 downto 0);
end record;

type passthrough_stub_out_pass_up_type is record
  ready : std_logic;
end record;

component passthrough_stub
  port(
    clk : in std_logic;
    rst : in std_logic;
    in_pass_dn : in passthrough_stub_in_pass_dn_type;
    in_pass_up : out passthrough_stub_in_pass_up_type;
    in_pass2_dn : in passthrough_stub_in_pass2_dn_type;
    in_pass2_up : out passthrough_stub_in_pass2_up_type;
    out_pass_dn : out passthrough_stub_out_pass_dn_type;
    out_pass_up : in passthrough_stub_out_pass_up_type
  );
end component;

component invalid_stub_com
  port(
    clk : in std_logic;
    rst : in std_logic
  );
end component;

component invalid_stub
  port(
    clk : in std_logic;
    rst : in std_logic
  );
end component;

component source_stub_com
  port(
    clk : in std_logic;
    rst : in std_logic;
    out_source_valid : out std_logic;
    out_source_ready : in std_logic;
    out_source_data : out std_logic_vector(263 downto 0);
    out_source_stai : out std_logic_vector(2 downto 0);
    out_source_endi : out std_logic_vector(2 downto 0);
    out_source_strb : out std_logic_vector(7 downto 0);
    out_source2_valid : out std_logic;
    out_source2_ready : in std_logic;
    out_source2_data : out std_logic_vector(7 downto 0);
    out_source2_stai : out std_logic_vector(2 downto 0);
    out_source2_endi : out std_logic_vector(2 downto 0);
    out_source2_strb : out std_logic_vector(7 downto 0);
    out_source2_a_valid : out std_logic;
    out_source2_a_ready : in std_logic;
    out_source2_a_data : out std_logic_vector(2047 downto 0);
    out_source2_a_stai : out std_logic_vector(5 downto 0);
    out_source2_a_endi : out std_logic_vector(5 downto 0);
    out_source2_a_strb : out std_logic_vector(63 downto 0);
    out_source2_b_valid : out std_logic;
    out_source2_b_ready : in std_logic;
    out_source2_b_data : out std_logic_vector(191 downto 0);
    out_source2_b_stai : out std_logic_vector(4 downto 0);
    out_source2_b_endi : out std_logic_vector(4 downto 0);
    out_source2_b_strb : out std_logic_vector(23 downto 0)
  );
end component;

type source_stub_out_source_data_type is record
  -- Variants: a, b
  tag : std_logic_vector(0 downto 0);
  a : std_logic_vector(31 downto 0);
  b : std_logic_vector(7 downto 0);
end record;

type source_stub_out_source_data_array_type is array (0 to 7) of source_stub_out_source_data_type;

type source_stub_out_source_dn_type is record
  valid : std_logic;
  data : source_stub_out_source_data_array_type;
  stai : std_logic_vector(2 downto 0);
  endi : std_logic_vector(2 downto 0);
  strb : std_logic_vector(7 downto 0);
end record;

type source_stub_out_source_up_type is record
  ready : std_logic;
end record;

type source_stub_out_source2_data_type is record
  -- Variants: a, b
  tag : std_logic_vector(0 downto 0);
end record;

type source_stub_out_source2_data_array_type is array (0 to 7) of source_stub_out_source2_data_type;

type source_stub_out_source2_dn_type is record
  valid : std_logic;
  data : source_stub_out_source2_data_array_type;
  stai : std_logic_vector(2 downto 0);
  endi : std_logic_vector(2 downto 0);
  strb : std_logic_vector(7 downto 0);
end record;

type source_stub_out_source2_up_type is record
  ready : std_logic;
end record;

type source_stub_out_source2_a_data_array_type is array (0 to 63) of std_logic_vector(31 downto 0);

type source_stub_out_source2_a_dn_type is record
  valid : std_logic;
  data : source_stub_out_source2_a_data_array_type;
  stai : std_logic_vector(5 downto 0);
  endi : std_logic_vector(5 downto 0);
  strb : std_logic_vector(63 downto 0);
end record;

type source_stub_out_source2_a_up_type is record
  ready : std_logic;
end record;

type source_stub_out_source2_b_data_array_type is array (0 to 23) of std_logic_vector(7 downto 0);

type source_stub_out_source2_b_dn_type is record
  valid : std_logic;
  data : source_stub_out_source2_b_data_array_type;
  stai : std_logic_vector(4 downto 0);
  endi : std_logic_vector(4 downto 0);
  strb : std_logic_vector(23 downto 0);
end record;

type source_stub_out_source2_b_up_type is record
  ready : std_logic;
end record;

component source_stub
  port(
    clk : in std_logic;
    rst : in std_logic;
    out_source_dn : out source_stub_out_source_dn_type;
    out_source_up : in source_stub_out_source_up_type;
    out_source2_dn : out source_stub_out_source2_dn_type;
    out_source2_up : in source_stub_out_source2_up_type;
    out_source2_a_dn : out source_stub_out_source2_a_dn_type;
    out_source2_a_up : in source_stub_out_source2_a_up_type;
    out_source2_b_dn : out source_stub_out_source2_b_dn_type;
    out_source2_b_up : in source_stub_out_source2_b_up_type
  );
end component;

component sink_stub_com
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

type sink_stub_in_sink_data_type is record
  -- Variants: a, b
  tag : std_logic_vector(0 downto 0);
  a : std_logic_vector(31 downto 0);
  b : std_logic_vector(7 downto 0);
end record;

type sink_stub_in_sink_data_array_type is array (0 to 7) of sink_stub_in_sink_data_type;

type sink_stub_in_sink_dn_type is record
  valid : std_logic;
  data : sink_stub_in_sink_data_array_type;
  stai : std_logic_vector(2 downto 0);
  endi : std_logic_vector(2 downto 0);
  strb : std_logic_vector(7 downto 0);
end record;

type sink_stub_in_sink_up_type is record
  ready : std_logic;
end record;

component sink_stub
  port(
    clk : in std_logic;
    rst : in std_logic;
    in_sink_dn : in sink_stub_in_sink_dn_type;
    in_sink_up : out sink_stub_in_sink_up_type
  );
end component;

end test_library;