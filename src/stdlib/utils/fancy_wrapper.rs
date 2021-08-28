use indexmap::IndexMap;

use crate::{
    cat,
    design::{implementation::composer::GenericComponent, Library, Streamlet, StreamletKey},
    generator::common::{
        convert::{Componentify, CANON_SUFFIX},
        Package,
    },
    stdlib::common::{
        architecture::{
            assignment::Assign, declaration::ObjectDeclaration, statement::PortMapping,
            Architecture,
        },
        entity::Entity,
    },
    Error, Result,
};

fn generate_fancy_wrapper<'a>(
    library: &Library,
    package: &'a Package,
    streamlet_key: &StreamletKey,
) -> Result<Architecture<'a>> {
    let streamlet = library.get_streamlet(streamlet_key.clone())?;
    let mut architecture =
        Architecture::new_default(package, cat!(streamlet_key, CANON_SUFFIX.unwrap()))?;
    let mut portmap =
        PortMapping::from_component(&package.get_component(streamlet_key.clone())?, "fancy")?;
    portmap
        .map_port(
            "clk",
            architecture
                .entity_ports()?
                .get("clk")
                .ok_or(Error::BackEndError(
                    "Entity does not have a clk signal".to_string(),
                ))?,
        )?
        .map_port(
            "rst",
            architecture
                .entity_ports()?
                .get("rst")
                .ok_or(Error::BackEndError(
                    "Entity does not have a rst signal".to_string(),
                ))?,
        )?;
    let mut fancy_wires = IndexMap::new();
    let mut fancy_assigns = vec![];
    let mut fixed_assign = |signal: &ObjectDeclaration, port_name: &str| -> Result<()> {
        fancy_assigns.push(
            signal.assign(architecture.entity_ports()?.get(port_name).ok_or(
                Error::BackEndError(format!("Entity does not have a {} signal", port_name)),
            )?)?,
        );
        Ok(())
    };
    for (port_name, object) in portmap.ports() {
        let signal = ObjectDeclaration::signal(cat!(port_name, "wire"), object.typ().clone(), None);
        if port_name == "clk" || port_name == "rst" {
            fixed_assign(&signal, port_name)?;
        }
        fancy_wires.insert(port_name.to_string(), signal);
    }
    for (port_name, wire) in fancy_wires {
        // TODO: Figure out how to relate the "fancy" ports back to the "canonical" ports and vice versa.
        portmap.map_port(port_name, &wire)?;
        architecture.add_declaration(wire)?;
    }
    for assign in fancy_assigns {
        architecture.add_statement(assign)?;
    }
    architecture.add_statement(portmap)?;

    Ok(architecture)
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use crate::{
        generator::{common::convert::Packify, vhdl::Declare},
        stdlib::basic::stub::tests::parsed_stub_project,
        Name,
    };

    use super::*;

    // Play around here
    #[test]
    fn print_fancy_wrapper() -> Result<()> {
        let lib_key = Name::try_from("test_library")?;
        let prj = parsed_stub_project()?;
        let lib = prj.get_lib(lib_key.clone())?;
        let pak = lib.fancy();
        let arch = generate_fancy_wrapper(lib, &pak, &StreamletKey::try_from("passthrough_stub")?)?;
        print!("{}", arch.declare()?);
        Ok(())
    }

    #[test]
    fn test_fancy_wrapper() -> Result<()> {
        let lib_key = Name::try_from("test_library")?;
        let prj = parsed_stub_project()?;
        let lib = prj.get_lib(lib_key.clone())?;
        let pak = lib.fancy();
        let arch = generate_fancy_wrapper(lib, &pak, &StreamletKey::try_from("passthrough_stub")?)?;
        assert_eq!(
            r#"library ieee;
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
   signal in_pass_wire : passthrough_stub_in_pass_type;
   signal in_pass2_wire : passthrough_stub_in_pass2_type;
   signal out_pass_wire : passthrough_stub_out_pass_type;
begin
--<architecture_statement_part>
   clk_wire <= clk;
   rst_wire <= rst;
   fancy: passthrough_stub port map(
     clk => clk_wire,
     rst => rst_wire,
     in_pass => in_pass_wire,
     in_pass2 => in_pass2_wire,
     out_pass => out_pass_wire
   );
end Behavioral;
"#,
            arch.declare()?
        );
        Ok(())
    }
}
