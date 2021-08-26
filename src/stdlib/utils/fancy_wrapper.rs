use crate::{Error, Result, cat, design::{Library, Streamlet, StreamletKey}, generator::common::{Package, convert::{CANON_SUFFIX, Componentify}}, stdlib::common::{architecture::{Architecture, statement::PortMapping}, entity::Entity}};

fn generate_fancy_wrapper<'a>(library: &Library, package: &'a Package, streamlet_key: &StreamletKey) -> Result<Architecture<'a>> {
    let streamlet = library.get_streamlet(streamlet_key.clone())?;
    let mut architecture = Architecture::new_default(package, cat!(streamlet_key, CANON_SUFFIX.unwrap()))?;
    let mut portmap = PortMapping::from_component(&package.get_component(streamlet_key.clone())?, "fancy")?;
    Ok(architecture)
}
