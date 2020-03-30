//! Structural implementations of streamlets.

use crate::design::structural::streamlet_instance::StreamletInst;
use crate::design::{InterfaceKey, StreamletRef};
use crate::Name;
use crate::{Error, Result};
use indexmap::map::IndexMap;
use std::fmt::Debug;

pub mod builder;
pub mod streamlet_instance;

pub trait IORef {
    fn io(&self, key: InterfaceKey) -> Result<NodeIORef>;
}

/// The key of an instance.
pub type NodeKey = Name;

impl NodeKey {
    /// Returns the key that signifies the streamlet that is being implemented itself.
    /// This is a reserved key that users should not be able to use as instance name.
    /// Could be seen as the Rust keyword "self".
    pub fn this() -> NodeKey {
        Name::try_new("this").unwrap()
    }
}

/// A reference to a structural node IO. Only valid within the context of an implementation.
#[derive(Clone, PartialEq)]
pub struct NodeIORef {
    node: NodeKey,
    interface: InterfaceKey,
}

impl NodeIORef {
    pub fn node(&self) -> NodeKey {
        self.node.clone()
    }

    pub fn interface(&self) -> InterfaceKey {
        self.interface.clone()
    }
}

/// A connection between two streamlets.
#[derive(Clone, PartialEq)]
pub struct Edge {
    /// A reference to the sourcing node interface.
    source: NodeIORef,
    /// A reference to the sinking node interface.
    sink: NodeIORef,
}

impl Edge {
    pub fn source(&self) -> NodeIORef {
        self.source.clone()
    }

    pub fn sink(&self) -> NodeIORef {
        self.source.clone()
    }
}

/// A node in the structural implementation graph.
#[derive(Clone, PartialEq)]
pub enum Node {
    // Slices, syncs etc.. can be added here.
    // this could be nicely abstracted with some required traits, so the impl of node
    // doesn't have to be as verbose as below
    This(StreamletInst),
    Streamlet(StreamletInst),
    // Sync(...)?
    // Slice(...)?
    // Buffer(...)?
    // Reshaper(...)?
}

impl Node {
    /// Return the key of this node.
    pub fn key(&self) -> NodeKey {
        match self {
            Node::This(_) => NodeKey::this(),
            Node::Streamlet(s) => s.key(),
        }
    }
}

/// A structural implementation of a streamlet.
///
/// The structural implementation is a simple graph with instances (nodes) and
/// connections (edges).
///
/// The edges weights contain only a node key and an interface key that reference the sinking
/// and sourcing node and interfaces.
///
/// The nodes are named by some key, this could be seen as the instance name or the reference
/// designator. Nodes can have various types, with currently two variants only; the "this"
/// node, referring to a streamlets external interfaces, and a streamlet instance node.
#[derive(Clone, PartialEq)]
pub struct StructuralImpl {
    streamlet: StreamletRef,
    nodes: IndexMap<NodeKey, Node>,
    edges: Vec<Edge>,
}

impl StructuralImpl {
    pub fn streamlet(&self) -> StreamletRef {
        self.streamlet.clone()
    }

    pub fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.nodes.iter().map(|(_, v)| v)
    }

    pub fn edges(&self) -> impl Iterator<Item = &Edge> {
        self.edges.iter()
    }

    pub fn node(&self, key: NodeKey) -> Result<Node> {
        Ok(self
            .nodes
            .get(&key)
            .ok_or_else(|| {
                Error::ImplementationError(format!(
                    "Structural implementation of streamlet {} has no node named {}.",
                    self.streamlet(),
                    key
                ))
            })?
            .clone())
    }
}

impl Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Node::This(s) => write!(f, "[This] {:?}", s),
            Node::Streamlet(s) => write!(f, "[Inst] {:?}", s),
        }
    }
}

impl Debug for Edge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} <-- {:?}", self.sink, self.source)
    }
}

impl Debug for NodeIORef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.node, self.interface)
    }
}

impl Debug for StructuralImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Streamlet: {}\nInstances:\n{}\nConnections:\n{}",
            self.streamlet,
            self.nodes
                .iter()
                .map(|(_, i)| format!("  {:?}", i))
                .collect::<Vec<String>>()
                .join("\n"),
            self.edges
                .iter()
                .map(|c| format!("  {:?}", c))
                .collect::<Vec<String>>()
                .join("\n"),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::design::implementation::Implementation;
    use crate::Identify;
    use builder::tests::builder_example;
    use std::ops::Deref;

    /// Demonstration of how to inspect a Tydi project.
    #[test]
    fn inspect() -> Result<()> {
        let prj = builder_example()?;

        println!("Project: {}", prj.identifier());

        for lib in prj.libraries() {
            println!("  Library: {}", lib.identifier());

            // Print named types:
            if lib.named_types().peekable().peek().is_some() {
                println!("    Named types:");
                for typ in lib.named_types() {
                    println!("      {} = {:?}", typ.key(), typ.logical())
                }
            }

            // Print streamlets:
            println!("    Streamlets:");
            for stl in lib.streamlets() {
                println!("      {}", stl.identifier());
                for io in stl.interfaces() {
                    println!("        {} : {} {}", io.key(), io.mode(), io.typ())
                }

                match stl.implementation().deref() {
                    Implementation::None => println!("      No implementation."),
                    Implementation::Structural(structural) => {
                        println!("      Structural implementation:");
                        println!("        Instances:");
                        for node in structural.nodes() {
                            match node {
                                Node::This(_) => (),
                                Node::Streamlet(inst) => {
                                    println!("          {} : {}", inst.key(), inst.streamlet());
                                    // Getting a real streamlet reference:
                                    prj.get_streamlet(inst.streamlet())?;
                                }
                            }
                        }

                        println!("        Edges:");
                        for edge in structural.edges() {
                            println!("          {:?}", edge);

                            // Inspecting edges is a bit involved at the moment.
                            // To get to the interface type of the source, for example:
                            let src = edge.source();
                            prj.get_streamlet(
                                match structural.node(src.node())? {
                                    Node::This(s) => s,
                                    Node::Streamlet(s) => s,
                                }
                                .streamlet(),
                            )?
                            .get_interface(src.interface())?;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
