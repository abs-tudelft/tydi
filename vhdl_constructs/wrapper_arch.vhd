library ieee;
use ieee.std_logic_1164.all;

library work;
use work.test_library.all;

entity passthrough_stub_com is
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
end passthrough_stub_com;

architecture wrapper of passthrough_stub_com is

  signal passthrough_stub_in_pass_data_array_wire  : passthrough_stub_in_pass_data_array_type;
  signal passthrough_stub_out_pass_data_array_wire : passthrough_stub_out_pass_data_array_type;
  signal some_std_r : std_logic_vector(3 downto 0);
  signal some_std_l : std_logic_vector(3 downto 0) := (3 downto 2 => some_std_r(3 downto 2), 1 downto 0 => some_std_r(1 downto 0));
begin

  passthrough_stub_in_pass_data_array_wire_map : for i in passthrough_stub_in_pass_data_array_type'range generate
    passthrough_stub_in_pass_data_array_wire(i) <= (
    tag => in_pass_data((i + 1) * 32 - 1 downto (i + 1) * 32 - 1),
    a   => in_pass_data((i + 1) * 32 - 2 downto i * 32),
    b   => in_pass_data((i + 1) * 32 - 26 downto i * 32)
    );
  end generate;

  passthrough_stub_out_pass_data_array_wire_map : for i in passthrough_stub_out_pass_data_array_type'range generate
    out_pass_data((i + 1) * 32 - 1 downto (i + 1) * 32 - 1) <= passthrough_stub_out_pass_data_array_wire(i).tag;
    out_pass_data((i + 1) * 32 - 2 downto i * 32)           <= passthrough_stub_out_pass_data_array_wire(i).a or passthrough_stub_out_pass_data_array_wire(i).b;
  end generate;

  wrap : passthrough_stub port map(
    clk                  => clk,
    rst                  => rst,
    in_pass_dn.valid     => in_pass_valid,
    in_pass_dn.data      => passthrough_stub_in_pass_data_array_wire,
    in_pass_dn.stai      => in_pass_stai,
    in_pass_dn.endi      => in_pass_endi,
    in_pass_dn.strb      => in_pass_strb,
    in_pass_up.ready     => in_pass_ready,
    in_pass2_dn.valid    => in_pass2_valid,
    in_pass2_dn.data.op1 => in_pass2_data(117 downto 54),
    in_pass2_dn.data.op2 => in_pass2_data(53 downto 0),
    in_pass2_dn.strb     => in_pass2_strb,
    in_pass2_up.ready    => in_pass2_ready,
    out_pass_dn.valid    => out_pass_valid,
    out_pass_dn.data     => passthrough_stub_out_pass_data_array_wire,
    out_pass_dn.stai     => out_pass_stai,
    out_pass_dn.endi     => out_pass_endi,
    out_pass_dn.strb     => out_pass_strb,
    out_pass_up.ready    => out_pass_ready
  );

end architecture;