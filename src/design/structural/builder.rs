//! Contains constructs to support building up structural implementations in Rust.
//!
//! This is useful until there is a front-end language for structural implementation.

use crate::design::structural::streamlet_instance::StreamletInst;
use crate::design::structural::{Edge, Node, NodeIORef, NodeKey, StructuralImpl};
use crate::design::{InterfaceKey, Project, StreamletRef};
use crate::{Error, Result};
use indexmap::map::IndexMap;
use std::convert::TryInto;

/// A view of a node
pub struct NodeView {
    node: Node,
}

/// A view of a node's IO
pub struct NodeIOView {
    reference: NodeIORef,
}

impl NodeView {
    pub fn io<K>(&self, key: K) -> Result<NodeIOView>
    where
        K: TryInto<InterfaceKey>,
        <K as TryInto<InterfaceKey>>::Error: Into<Error>,
    {
        let k = key.try_into().map_err(Into::into)?;
        Ok(NodeIOView {
            reference: NodeIORef {
                node: self.node.key(),
                interface: k,
            },
        })
    }
}

impl NodeIOView {
    pub fn key(&self) -> String {
        format!("{}.{}", self.reference.node, self.reference.interface)
    }

    pub fn to_ref(&self) -> NodeIORef {
        self.reference.clone()
    }
}

/// The StructuralImplBuilder struct allows users to implement streamlets by structurally
/// combining streamlets into larger designs.
#[derive(Clone, PartialEq)]
pub struct StructuralImplBuilder<'prj> {
    project: &'prj Project,
    streamlet: StreamletRef,
    instances: IndexMap<NodeKey, Node>,
    connections: Vec<Edge>,
}

impl<'prj> StructuralImplBuilder<'prj> {
    pub fn try_new(project: &'prj Project, streamlet: StreamletRef) -> Result<Self> {
        // Check if the reference is OK.
        project.get_streamlet(streamlet.clone())?;
        // Return a new empty structural impl.
        Ok(StructuralImplBuilder {
            project,
            streamlet: streamlet.clone(),
            instances: vec![(
                NodeKey::this(),
                Node::This(StreamletInst::new(NodeKey::this(), streamlet)),
            )]
            .into_iter()
            .collect::<IndexMap<NodeKey, Node>>(),
            connections: Vec::new(),
        })
    }

    pub fn finish(self) -> StructuralImpl {
        StructuralImpl {
            streamlet: self.streamlet,
            nodes: self.instances,
            edges: self.connections,
        }
    }

    // HCL
    pub fn instantiate<I>(&mut self, streamlet: StreamletRef, instance: I) -> Result<NodeView>
    where
        I: TryInto<NodeKey>,
        <I as TryInto<NodeKey>>::Error: Into<Error>,
    {
        let key = instance.try_into().map_err(Into::into)?;
        if self.instances.get(&key).is_some() {
            Err(Error::ImplementationError(format!(
                "Instance {} already exists in structural implementation of {:?}",
                key, streamlet
            )))
        } else {
            // Set up a node.
            let node = Node::Streamlet(StreamletInst::new(key.clone(), streamlet));
            // Copy and insert the node.
            self.instances.insert(key, node.clone());
            // Return a structural node reference with a copy of the node.
            Ok(NodeView {
                //project: Clone::clone(&self.project),
                node,
            })
        }
    }

    // HCL
    pub fn this(&self) -> NodeView {
        NodeView {
            //project: self.project,
            // The this instance should always exist, so it is safe to unwrap here.
            node: self.instances.get(&NodeKey::this()).unwrap().clone(),
        }
    }

    // HCL
    pub fn connect(&mut self, source: Result<NodeIOView>, sink: Result<NodeIOView>) -> Result<()> {
        let source = source?;
        let sink = sink?;
        self.connections.push(Edge {
            source: source.to_ref(),
            sink: sink.to_ref(),
        });
        Ok(())
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::design::implementation::Implementation;
    use crate::design::structural::builder::StructuralImplBuilder;
    use crate::design::{
        Interface, Library, LibraryKey, Mode, NamedType, Project, Streamlet, TypeRef,
    };
    use crate::logical::LogicalType;
    use crate::{Name, UniqueKeyBuilder};

    pub(in crate::design) fn builder_example() -> Result<Project> {
        ////////////////////////////////////////////////////////////////////////////////////////////

        // Declare a lib for primitive streamlets.
        let mut prim = Library::new(LibraryKey::try_new("primitives")?);

        // Add the types to the library:
        let wheat = prim.add_type(NamedType::try_new("Wheat", LogicalType::Null)?)?;
        let flour = prim.add_type(NamedType::try_new("Flour", LogicalType::Null)?)?;
        // add_type returns a custom type reference that can only be de-referenced through a
        //  project but can be used in things that reference this type.
        // For users, there should be no other way of obtaining this custom reference.

        // Add streamlet to lib.
        let windmill = prim.add_streamlet(Streamlet::from_builder(
            "Windmill",
            UniqueKeyBuilder::new().with_items(vec![
                Interface::try_new("wheat", Mode::In, wheat.clone(), None)?,
                Interface::try_new("flour", Mode::Out, flour.clone(), None)?,
            ]),
            None,
        )?)?;

        // Add another type and streamlet.
        let cookie = prim.add_type(NamedType::try_new("Cookie", LogicalType::Null)?)?;
        let bakery = prim.add_streamlet(Streamlet::from_builder(
            "Bakery",
            UniqueKeyBuilder::new().with_items(vec![
                // Using the Flour type from another library.
                Interface::try_new("flour", Mode::In, flour, None)?,
                Interface::try_new("cookies", Mode::Out, cookie.clone(), None)?,
            ]),
            None,
        )?)?;

        let mut factories = Library::new(LibraryKey::try_new("factories")?);
        let cookie_factory = factories.add_streamlet(Streamlet::from_builder(
            "Factory",
            UniqueKeyBuilder::new().with_items(vec![
                Interface::try_new("wheat", Mode::In, wheat, None)?,
                // Some unnamed secret ingredient to make the cookies taste good.
                Interface::try_new("secret", Mode::In, TypeRef::anon(LogicalType::Null), None)?,
                Interface::try_new("cookies", Mode::Out, cookie, None)?,
            ]),
            None,
        )?)?;

        // Set up a project and add libraries:
        let mut prj = Project::new(Name::try_new("test")?);
        prj.add_library(prim)?;
        prj.add_library(factories)?;

        ////////////////////////////////////////////////////////////////////////////////////////////

        // Set up an implementation for the cookie factory.
        let mut imp = StructuralImplBuilder::try_new(&prj, cookie_factory.clone())?;

        let this = imp.this();
        let mill = imp.instantiate(windmill, "mill")?;
        let baker = imp.instantiate(bakery, "baker")?;

        imp.connect(this.io("wheat"), mill.io("wheat"))?;
        imp.connect(mill.io("flour"), baker.io("flour"))?;
        imp.connect(baker.io("cookies"), this.io("cookies"))?;

        let imp = imp.finish();

        // dbg!(&imp);
        prj.add_streamlet_impl(cookie_factory, Implementation::Structural(imp))?;

        Ok(prj)
    }
}
