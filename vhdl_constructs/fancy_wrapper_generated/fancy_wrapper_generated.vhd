library ieee;
use ieee.std_logic_1164.all;

library work;
use work.test_library.all;

entity passthrough_stub_com is
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
end passthrough_stub_com;

architecture Behavioral of passthrough_stub_com is
--<architecture_declarative_part>
   signal clk_wire : std_logic;
   signal rst_wire : std_logic;
   signal in_pass_dn_wire : passthrough_stub_in_pass_dn_type;
   signal in_pass_up_wire : passthrough_stub_in_pass_up_type;
   signal in_pass2_dn_wire : passthrough_stub_in_pass2_dn_type;
   signal in_pass2_up_wire : passthrough_stub_in_pass2_up_type;
   signal out_pass_dn_wire : passthrough_stub_out_pass_dn_type;
   signal out_pass_up_wire : passthrough_stub_out_pass_up_type;
begin
--<architecture_statement_part>
   clk_wire <= clk;
   rst_wire <= rst;
   in_pass_dn_wire.valid <= in_pass_valid;
   in_pass_dn_wire.data(0).tag <= in_pass_data(32 downto 32);
   in_pass_dn_wire.data(0).a <= in_pass_data(31 downto 0);
   in_pass_dn_wire.data(0).b <= in_pass_data(7 downto 0);
   in_pass_dn_wire.data(1).tag <= in_pass_data(65 downto 65);
   in_pass_dn_wire.data(1).a <= in_pass_data(64 downto 33);
   in_pass_dn_wire.data(1).b <= in_pass_data(40 downto 33);
   in_pass_dn_wire.data(2).tag <= in_pass_data(98 downto 98);
   in_pass_dn_wire.data(2).a <= in_pass_data(97 downto 66);
   in_pass_dn_wire.data(2).b <= in_pass_data(73 downto 66);
   in_pass_dn_wire.data(3).tag <= in_pass_data(131 downto 131);
   in_pass_dn_wire.data(3).a <= in_pass_data(130 downto 99);
   in_pass_dn_wire.data(3).b <= in_pass_data(106 downto 99);
   in_pass_dn_wire.data(4).tag <= in_pass_data(164 downto 164);
   in_pass_dn_wire.data(4).a <= in_pass_data(163 downto 132);
   in_pass_dn_wire.data(4).b <= in_pass_data(139 downto 132);
   in_pass_dn_wire.data(5).tag <= in_pass_data(197 downto 197);
   in_pass_dn_wire.data(5).a <= in_pass_data(196 downto 165);
   in_pass_dn_wire.data(5).b <= in_pass_data(172 downto 165);
   in_pass_dn_wire.data(6).tag <= in_pass_data(230 downto 230);
   in_pass_dn_wire.data(6).a <= in_pass_data(229 downto 198);
   in_pass_dn_wire.data(6).b <= in_pass_data(205 downto 198);
   in_pass_dn_wire.data(7).tag <= in_pass_data(263 downto 263);
   in_pass_dn_wire.data(7).a <= in_pass_data(262 downto 231);
   in_pass_dn_wire.data(7).b <= in_pass_data(238 downto 231);
   in_pass_dn_wire.stai <= in_pass_stai;
   in_pass_dn_wire.endi <= in_pass_endi;
   in_pass_dn_wire.strb <= in_pass_strb;
   in_pass_ready <= in_pass_up_wire.ready;
   in_pass2_dn_wire.valid <= in_pass2_valid;
   in_pass2_dn_wire.data.op1 <= in_pass2_data(63 downto 0);
   in_pass2_dn_wire.data.op2 <= in_pass2_data(117 downto 64);
   in_pass2_dn_wire.strb <= in_pass2_strb;
   in_pass2_ready <= in_pass2_up_wire.ready;
   out_pass_valid <= out_pass_dn_wire.valid;
   out_pass_data(32 downto 32) <= out_pass_dn_wire.data(0).tag;
   out_pass_data(31 downto 0) <= out_pass_dn_wire.data(0).a;
   out_pass_data(7 downto 0) <= out_pass_dn_wire.data(0).b;
   out_pass_data(65 downto 65) <= out_pass_dn_wire.data(1).tag;
   out_pass_data(64 downto 33) <= out_pass_dn_wire.data(1).a;
   out_pass_data(40 downto 33) <= out_pass_dn_wire.data(1).b;
   out_pass_data(98 downto 98) <= out_pass_dn_wire.data(2).tag;
   out_pass_data(97 downto 66) <= out_pass_dn_wire.data(2).a;
   out_pass_data(73 downto 66) <= out_pass_dn_wire.data(2).b;
   out_pass_data(131 downto 131) <= out_pass_dn_wire.data(3).tag;
   out_pass_data(130 downto 99) <= out_pass_dn_wire.data(3).a;
   out_pass_data(106 downto 99) <= out_pass_dn_wire.data(3).b;
   out_pass_data(164 downto 164) <= out_pass_dn_wire.data(4).tag;
   out_pass_data(163 downto 132) <= out_pass_dn_wire.data(4).a;
   out_pass_data(139 downto 132) <= out_pass_dn_wire.data(4).b;
   out_pass_data(197 downto 197) <= out_pass_dn_wire.data(5).tag;
   out_pass_data(196 downto 165) <= out_pass_dn_wire.data(5).a;
   out_pass_data(172 downto 165) <= out_pass_dn_wire.data(5).b;
   out_pass_data(230 downto 230) <= out_pass_dn_wire.data(6).tag;
   out_pass_data(229 downto 198) <= out_pass_dn_wire.data(6).a;
   out_pass_data(205 downto 198) <= out_pass_dn_wire.data(6).b;
   out_pass_data(263 downto 263) <= out_pass_dn_wire.data(7).tag;
   out_pass_data(262 downto 231) <= out_pass_dn_wire.data(7).a;
   out_pass_data(238 downto 231) <= out_pass_dn_wire.data(7).b;
   out_pass_stai <= out_pass_dn_wire.stai;
   out_pass_endi <= out_pass_dn_wire.endi;
   out_pass_strb <= out_pass_dn_wire.strb;
   out_pass_up_wire.ready <= out_pass_ready;
   fancy: passthrough_stub port map(
     clk => clk_wire,
     rst => rst_wire,
     in_pass_dn => in_pass_dn_wire,
     in_pass_up => in_pass_up_wire,
     in_pass2_dn => in_pass2_dn_wire,
     in_pass2_up => in_pass2_up_wire,
     out_pass_dn => out_pass_dn_wire,
     out_pass_up => out_pass_up_wire
   );
end Behavioral;