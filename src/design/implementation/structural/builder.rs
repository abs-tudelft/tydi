//! Contains constructs to support building up structural implementations in Rust.
//!
//! This is useful until there is a front-end language for structural implementation.

use crate::design::implementation::structural::{
    Edge, Node, NodeIORef, NodeKey, StreamletInst, StructuralImpl,
};
use crate::design::{Interface, InterfaceKey, Mode, Project, StreamletRef};
use crate::{Error, Result};
use indexmap::map::IndexMap;
use std::convert::TryInto;

/// Trait to construct node interface references from various node types.
pub trait Interfaces {
    fn io<K>(&self, key: K) -> Result<NodeIORef>
    where
        K: TryInto<InterfaceKey>,
        <K as TryInto<InterfaceKey>>::Error: Into<Error>;
}

impl Interfaces for Node {
    /// Obtain a reference to a node interface.
    ///
    /// # Example:
    ///
    /// ```rust
    /// use tydi::design::implementation::prelude::*;
    /// use tydi::Result;
    /// # fn example() -> Result<()> {
    ///
    /// let mut prj = Project::new(Name::try_new("example")?);
    /// let mut lib = Library::new(Name::try_new("lib")?);
    /// let foo = lib.add_streamlet(Streamlet::try_new(
    ///     "roast",
    ///     vec![
    ///         Interface::try_new("beans", Mode::In, TypeRef::anon(LogicalType::Null), None)?,
    ///         Interface::try_new("coffee", Mode::Out, TypeRef::anon(LogicalType::Null), None)?,
    ///     ],
    ///     None,
    /// )?)?;
    ///
    /// let mut builder = StructuralImplBuilder::try_new(&prj, &foo)?;
    ///
    /// let beans = builder.this().io("beans"); // Obtain a (valid) reference to the beans interface.
    /// let coffee = builder.this().io("covfefe"); // Oops, made a typo! The reference is invalid.
    /// assert!(builder.connect(coffee, beans).is_err()); // Usage results in an error.
    ///
    /// // Fix our mistake:
    /// let beans = builder.this().io("beans");
    /// let coffee = builder.this().io("coffee");
    /// assert!(builder.connect(coffee, beans).is_ok());
    ///
    /// #   Ok(())
    /// # }
    /// ```
    ///
    /// # Considerations:
    ///
    /// The reference may be invalid, in the sense that it might not be checked whether
    /// the interface key is valid for that node, only if it's a valid InterfaceKey.
    ///
    /// Functions that take a NodeIORef as a parameter, will check for correctness of the
    /// reference, e.g. the connect() function of a StructuralImplBuilder.
    fn io<K>(&self, key: K) -> Result<NodeIORef>
    where
        K: TryInto<InterfaceKey>,
        <K as TryInto<InterfaceKey>>::Error: Into<Error>,
    {
        let k = key.try_into().map_err(Into::into)?;
        Ok(NodeIORef {
            node: self.key().clone(),
            interface: k,
        })
    }
}

/// The StructuralImplBuilder struct allows users to implement streamlets by structurally
/// combining streamlets into larger designs.
#[derive(Clone, PartialEq)]
pub struct StructuralImplBuilder<'p> {
    project: &'p Project<'p>,
    imp: StructuralImpl,
}

impl<'p> StructuralImplBuilder<'p> {
    /// Construct a new StructuralImplBuilder.
    ///
    /// This function returns an Error if the streamlet reference is invalid w.r.t. the project.
    pub fn try_new(project: &'p Project<'p>, streamlet: &StreamletRef) -> Result<Self> {
        // Check if the reference is OK.
        project.get_streamlet(streamlet)?;
        // Return a new empty structural impl.
        Ok(StructuralImplBuilder {
            project,
            imp: StructuralImpl {
                streamlet: streamlet.clone(),
                nodes: vec![(
                    NodeKey::this().clone(),
                    Node::This(StreamletInst::new(&NodeKey::this(), streamlet)),
                )]
                .into_iter()
                .collect::<IndexMap<NodeKey, Node>>(),
                edges: Vec::new(),
            },
        })
    }

    /// Finalize the builder, releasing the borrow to the project in which this implementation
    /// must reside.
    pub fn finish(self) -> StructuralImpl {
        self.imp
    }

    /// Instantiate a streamlet from a streamlet reference.
    pub fn instantiate<I>(&mut self, streamlet: StreamletRef, instance: I) -> Result<Node>
    where
        I: TryInto<NodeKey>,
        <I as TryInto<NodeKey>>::Error: Into<Error>,
    {
        let key = instance.try_into().map_err(Into::into)?;
        if self.imp.nodes.get(&key).is_some() {
            Err(Error::ImplementationError(format!(
                "Instance {} already exists in structural implementation of {:?}",
                key, streamlet
            )))
        } else {
            // Set up a node.
            let node = Node::Streamlet(StreamletInst::new(&key, &streamlet));
            // Copy and insert the node.
            self.imp.nodes.insert(key, node.clone());
            // Return a structural node reference with a copy of the node.
            Ok(node)
        }
    }

    /// Return the node representing the external interfaces of the streamlet itself.
    pub fn this(&self) -> Node {
        // We can unwrap safely here because the "this" node should always exist.
        self.imp.nodes.get(&NodeKey::this()).unwrap().clone()
    }

    fn get_interface(&self, io: NodeIORef) -> Result<Interface<'p>> {
        self.imp
            .node(io.node())?
            .get_interface(self.project, io.interface())
    }

    // Connect two streamlet interfaces.
    pub fn connect(&mut self, sink: Result<NodeIORef>, source: Result<NodeIORef>) -> Result<()> {
        // Check if the io references have been properly constructed.
        let sink = sink?;
        let source = source?;

        // Check if the references are valid, e.g. if an actual streamlet with those interface keys
        // exists in the project, and obtain references to the actual interfaces to check stuff.
        let sink_if = self.get_interface(sink.clone())?;
        let source_if = self.get_interface(source.clone())?;

        // Check interface compatibility.
        if source_if.mode() != Mode::Out {
            Err(Error::ImplementationError(format!(
                "Attempting to connect {:?} as source, but interface is not an output.",
                source
            )))
        } else if sink_if.mode() != Mode::In {
            Err(Error::ImplementationError(format!(
                "Attempting to connect {:?} as sink, but interface is not an input.",
                source
            )))
        } else if source_if.typ() != sink_if.typ() {
            Err(Error::ImplementationError(format!(
                "Types of sink {:?} : {:?}, and of source {:?} : {:?}, are incompatible.",
                sink,
                sink_if.typ(),
                source,
                source_if.typ()
            )))
        } else if self.imp.get_edge(source.clone()).is_some() {
            Err(Error::ImplementationError(format!(
                "Cannot connect sink {:?} to source {:?}, source is already connected.",
                sink, source
            )))
        } else if self.imp.get_edge(sink.clone()).is_some() {
            Err(Error::ImplementationError(format!(
                "Cannot connect sink {:?} to source {:?}, sink is already connected.",
                sink, source
            )))
        } else {
            self.imp.edges.push(Edge { source, sink });
            Ok(())
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    use crate::design::implementation::prelude::*;

    pub(crate) fn builder_example<'p>() -> Result<Project<'p>> {
        ////////////////////////////////////////////////////////////////////////////////////////////

        // Declare a lib for primitive streamlets.
        let mut prim = Library::new(LibraryKey::try_new("primitives")?);

        // Add the types to the library:
        let wheat = prim.add_type(NamedType::try_new("Wheat", LogicalType::Null, None)?)?;
        let flour = prim.add_type(NamedType::try_new("Flour", LogicalType::Null, None)?)?;
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
        let cookie = prim.add_type(NamedType::try_new("Cookie", LogicalType::Null, None)?)?;
        let bakery = prim.add_streamlet(Streamlet::from_builder(
            "Bakery",
            UniqueKeyBuilder::new().with_items(vec![
                // Using the Flour type from another library.
                Interface::try_new("flour", Mode::In, flour, None)?,
                // Some unnamed secret ingredient to make the cookies taste good.
                Interface::try_new("secret", Mode::In, LogicalType::Null, None)?,
                Interface::try_new("cookies", Mode::Out, cookie.clone(), None)?,
            ]),
            None,
        )?)?;

        let mut factories = Library::new(LibraryKey::try_new("factories")?);
        let cookie_factory = factories.add_streamlet(Streamlet::from_builder(
            "Factory",
            UniqueKeyBuilder::new().with_items(vec![
                Interface::try_new("wheat", Mode::In, wheat, None)?,
                Interface::try_new("secret", Mode::In, LogicalType::Null, None)?,
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
        let mut imp = StructuralImplBuilder::try_new(&prj, &cookie_factory)?;

        let this = imp.this();
        let mill = imp.instantiate(windmill, "mill")?;
        let baker = imp.instantiate(bakery, "baker")?;

        // TODO: confirm the correct error is produced not using dbg.
        // Attempting to sink an output.
        assert!(dbg!(imp.connect(mill.io("flour"), mill.io("flour"))).is_err());
        // Attempting to source an input.
        assert!(dbg!(imp.connect(baker.io("flour"), baker.io("flour"))).is_err());
        // Type conflict:
        assert!(dbg!(imp.connect(mill.io("wheat"), baker.io("cookies"))).is_err());

        imp.connect(mill.io("wheat"), this.io("wheat"))?;
        imp.connect(baker.io("secret"), this.io("secret"))?;
        imp.connect(baker.io("flour"), mill.io("flour"))?;
        imp.connect(this.io("cookies"), baker.io("cookies"))?;

        // Attempting to connect an io that is already connected:
        assert!(dbg!(imp.connect(mill.io("wheat"), this.io("wheat"))).is_err());

        let struct_imp = imp.finish();

        // dbg!(&imp);
        prj.add_streamlet_impl(&cookie_factory, Implementation::Structural(struct_imp))?;

        Ok(prj)
    }
}
