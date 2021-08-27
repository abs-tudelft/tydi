use crate::{Error, Result, cat, design::{Library, Streamlet, StreamletKey, implementation::composer::GenericComponent}, generator::common::{
        convert::{Componentify, CANON_SUFFIX},
        Package,
    }, stdlib::common::{
        architecture::{statement::PortMapping, Architecture},
        entity::Entity,
    }};

fn generate_fancy_wrapper<'a>(
    library: &Library,
    package: &'a Package,
    streamlet_key: &StreamletKey,
) -> Result<Architecture<'a>> {
    let streamlet = library.get_streamlet(streamlet_key.clone())?;
    let architecture =
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
    // TODO: Figure out how to relate the "fancy" ports back to the "canonical" ports and vice versa.
    Ok(architecture)
}
