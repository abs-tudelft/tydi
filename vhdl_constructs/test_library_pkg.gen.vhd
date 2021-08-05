library ieee;
use ieee.std_logic_1164.all;

package test_library is

component source_stub_com
  port(
    clk : in std_logic;
    rst : in std_logic;
    out_source_valid : out std_logic;
    out_source_ready : in std_logic;
    out_source_data : out std_logic_vector(263 downto 0)
  );
end component;

type source_stub_out_source_data_dn_type is record
  tag : std_logic_vector(0 downto 0);
  a : std_logic_vector(31 downto 0);
  b : std_logic_vector(31 downto 0);
end record;

type source_stub_out_source_dn_type is record
  valid : std_logic;
  data : source_stub_out_source_data_dn_type;
end record;

type source_stub_out_source_up_type is record
  ready : std_logic;
end record;

component source_stub
  port(
    clk : in std_logic;
    rst : in std_logic;
    out_source_dn : out source_stub_out_source_dn_type;
    out_source_up : in source_stub_out_source_up_type
  );
end component;

component sink_stub_com
  port(
    clk : in std_logic;
    rst : in std_logic;
    in_sink_valid : in std_logic;
    in_sink_ready : out std_logic;
    in_sink_data : in std_logic_vector(263 downto 0)
  );
end component;

type sink_stub_in_sink_data_dn_type_tag is (a, b);

type sink_stub_in_sink_data_dn_type is record
  tag : sink_stub_in_sink_data_dn_type_tag;
  data : std_logic_vector(31 downto 0);
end record;

type sink_stub_in_sink_dn_type is record
  valid : std_logic;
  data : sink_stub_in_sink_data_dn_type;
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

component source_com
  port(
    clk : in std_logic;
    rst : in std_logic;
    out_valid : out std_logic;
    out_ready : in std_logic;
    out_data : out std_logic_vector(263 downto 0)
  );
end component;

type source_out_data_dn_type_tag is (a, b);

type source_out_data_dn_type is record
  tag : source_out_data_dn_type_tag;
  data : std_logic_vector(31 downto 0);
end record;

type source_out_data_dn_type_a_range is range 15 downto 0; -- Hypothetical, if they're different ranges
type source_out_data_dn_type_b_range is range 31 downto 0;

type source_out_dn_type is record
  valid : std_logic;
  data : sink_stub_in_sink_data_dn_type;
end record;

type source_out_up_type is record
  ready : std_logic;
end record;

component source
  port(
    clk : in std_logic;
    rst : in std_logic;
    out_dn : out source_out_dn_type;
    out_up : in source_out_up_type
  );
end component;

component sink_com
  port(
    clk : in std_logic;
    rst : in std_logic;
    in_valid : in std_logic;
    in_ready : out std_logic;
    in_data : in std_logic_vector(263 downto 0)
  );
end component;

type sink_in_data_dn_type is record
  tag : std_logic_vector(0 downto 0);
  a : std_logic_vector(31 downto 0);
  b : std_logic_vector(31 downto 0);
end record;

type sink_in_dn_type is record
  valid : std_logic;
  data : sink_in_data_dn_type;
end record;

type sink_in_up_type is record
  ready : std_logic;
end record;

component sink
  port(
    clk : in std_logic;
    rst : in std_logic;
    in_dn : in sink_in_dn_type;
    in_up : out sink_in_up_type
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

component passthrough_com
  port(
    clk : in std_logic;
    rst : in std_logic;
    in_valid : in std_logic;
    in_ready : out std_logic;
    in_data : in std_logic_vector(263 downto 0);
    out_valid : out std_logic;
    out_ready : in std_logic;
    out_data : out std_logic_vector(263 downto 0)
  );
end component;

type passthrough_in_data_dn_type is record
  tag : std_logic_vector(0 downto 0);
  a : std_logic_vector(31 downto 0);
  b : std_logic_vector(31 downto 0);
end record;

type passthrough_in_dn_type is record
  valid : std_logic;
  data : passthrough_in_data_dn_type;
end record;

type passthrough_in_up_type is record
  ready : std_logic;
end record;

type passthrough_out_data_dn_type is record
  tag : std_logic_vector(0 downto 0);
  a : std_logic_vector(31 downto 0);
  b : std_logic_vector(31 downto 0);
end record;

type passthrough_out_dn_type is record
  valid : std_logic;
  data : passthrough_out_data_dn_type;
end record;

type passthrough_out_up_type is record
  ready : std_logic;
end record;

component passthrough
  port(
    clk : in std_logic;
    rst : in std_logic;
    in_dn : in passthrough_in_dn_type;
    in_up : out passthrough_in_up_type;
    out_dn : out passthrough_out_dn_type;
    out_up : in passthrough_out_up_type
  );
end component;

component passthrough_stub_com
  port(
    clk : in std_logic;
    rst : in std_logic;
    in_pass_valid : in std_logic;
    in_pass_ready : out std_logic;
    in_pass_data : in std_logic_vector(263 downto 0);
    out_pass_valid : out std_logic;
    out_pass_ready : in std_logic;
    out_pass_data : out std_logic_vector(263 downto 0)
  );
end component;

type passthrough_stub_in_pass_data_dn_type is record
  tag : std_logic_vector(0 downto 0);
  a : std_logic_vector(31 downto 0);
  b : std_logic_vector(31 downto 0);
end record;

type passthrough_stub_in_pass_dn_type is record
  valid : std_logic;
  data : passthrough_stub_in_pass_data_dn_type;
end record;

type passthrough_stub_in_pass_up_type is record
  ready : std_logic;
end record;

type passthrough_stub_out_pass_data_dn_type is record
  tag : std_logic_vector(0 downto 0);
  a : std_logic_vector(31 downto 0);
  b : std_logic_vector(31 downto 0);
end record;

type passthrough_stub_out_pass_dn_type is record
  valid : std_logic;
  data : passthrough_stub_out_pass_data_dn_type;
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
    out_pass_dn : out passthrough_stub_out_pass_dn_type;
    out_pass_up : in passthrough_stub_out_pass_up_type
  );
end component;

end test_library;