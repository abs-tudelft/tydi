library ieee;
use ieee.std_logic_1164.all;

package test_library is

  component passthrough_stub_com
    port (
      clk            : in std_logic;
      rst            : in std_logic;
      in_pass_valid  : in std_logic;
      in_pass_ready  : out std_logic;
      in_pass_data   : in std_logic_vector(263 downto 0);
      in_pass_stai   : in std_logic_vector(2 downto 0);
      in_pass_endi   : in std_logic_vector(2 downto 0);
      in_pass_strb   : in std_logic_vector(7 downto 0);
      in_pass2_valid : in std_logic;
      in_pass2_ready : out std_logic;
      in_pass2_data  : in std_logic_vector(117 downto 0);
      in_pass2_strb  : in std_logic_vector(0 downto 0);
      out_pass_valid : out std_logic;
      out_pass_ready : in std_logic;
      out_pass_data  : out std_logic_vector(263 downto 0);
      out_pass_stai  : out std_logic_vector(2 downto 0);
      out_pass_endi  : out std_logic_vector(2 downto 0);
      out_pass_strb  : out std_logic_vector(7 downto 0)
    );
  end component;

  type passthrough_stub_in_pass_data_type is record
    tag   : std_logic_vector(0 downto 0);
    union : std_logic_vector(31 downto 0);
  end record;

  type passthrough_stub_in_pass_data_array_type is array (0 to 7) of passthrough_stub_in_pass_data_type;

  type passthrough_stub_in_pass_dn_type is record
    valid : std_logic;
    data  : passthrough_stub_in_pass_data_array_type;
    stai  : std_logic_vector(2 downto 0);
    endi  : std_logic_vector(2 downto 0);
    strb  : std_logic_vector(7 downto 0);
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
    data  : passthrough_stub_in_pass2_data_dn_type;
    strb  : std_logic_vector(0 downto 0);
  end record;

  type passthrough_stub_in_pass2_up_type is record
    ready : std_logic;
  end record;

  type passthrough_stub_out_pass_data_type is record
    tag   : std_logic_vector(0 downto 0);
    union : std_logic_vector(31 downto 0);
  end record;

  type passthrough_stub_out_pass_data_array_type is array (0 to 7) of passthrough_stub_out_pass_data_type;

  type passthrough_stub_out_pass_dn_type is record
    valid : std_logic;
    data  : passthrough_stub_out_pass_data_array_type;
    stai  : std_logic_vector(2 downto 0);
    endi  : std_logic_vector(2 downto 0);
    strb  : std_logic_vector(7 downto 0);
  end record;

  type passthrough_stub_out_pass_up_type is record
    ready : std_logic;
  end record;

  component passthrough_stub
    port (
      clk         : in std_logic;
      rst         : in std_logic;
      in_pass_dn  : in passthrough_stub_in_pass_dn_type;
      in_pass_up  : out passthrough_stub_in_pass_up_type;
      in_pass2_dn : in passthrough_stub_in_pass2_dn_type;
      in_pass2_up : out passthrough_stub_in_pass2_up_type;
      out_pass_dn : out passthrough_stub_out_pass_dn_type;
      out_pass_up : in passthrough_stub_out_pass_up_type
    );
  end component;

end test_library;