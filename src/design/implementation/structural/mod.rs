//! Structural implementations of streamlets.

use crate::design::{Interface, InterfaceKey, Project, StreamletRef};
use crate::{Error, Result};
use crate::{Name, Reversed};
use indexmap::map::IndexMap;
use std::fmt::Debug;

pub mod builder;
pub mod instance;
pub use builder::Interfaces;
pub use builder::StructuralImplBuilder;
pub use instance::StreamletInst;

/// The key of an instance.
pub type NodeKey = Name;

pub const THIS_KEY: &str = "this";

impl NodeKey {
    /// Returns the key that signifies the streamlet that is being implemented itself.
    /// This is a reserved key that users should not be able to use as instance name.
    /// Could be seen as the Rust keyword "self".
    pub fn this() -> NodeKey {
        Name::try_new(THIS_KEY).unwrap()
    }
}

/// A reference to a structural node interface.
///
/// Only valid within the context of an implementation.
#[derive(Clone, PartialEq)]
pub struct NodeIORef {
    node: NodeKey,
    interface: InterfaceKey,
}

impl NodeIORef {
    /// Return the node key of this node interface reference.
    pub fn node(&self) -> NodeKey {
        self.node.clone()
    }

    /// Return the interface key of this node interface reference.
    pub fn interface(&self) -> InterfaceKey {
        self.interface.clone()
    }
}

/// A directed edge between two node interfaces.
#[derive(Clone, PartialEq)]
pub struct Edge {
    /// A reference to the sourcing node interface.
    source: NodeIORef,
    /// A reference to the sinking node interface.
    sink: NodeIORef,
}

impl Edge {
    /// Return the sourcing node interface of the edge.
    pub fn source(&self) -> NodeIORef {
        self.source.clone()
    }

    /// Return the sinking node interface of the edge.
    pub fn sink(&self) -> NodeIORef {
        self.sink.clone()
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

    /// Return a clone of the interface with some key on this node, in the context of a project.
    fn get_interface(&self, project: &Project, key: InterfaceKey) -> Result<Interface> {
        match self {
            // For the This node, we need to reverse the interface with respect to the streamlet
            // definition, because we are looking at this interface from the other side.
            Node::This(s) => s.get_interface(project, key).map(|i| i.reversed()),
            Node::Streamlet(s) => s.get_interface(project, key),
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
    /// Return a reference to the streamlet which is implemented by this implementation.
    pub fn streamlet(&self) -> StreamletRef {
        self.streamlet.clone()
    }

    /// Return an iterator over all nodes in the structural graph.
    pub fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.nodes.iter().map(|(_, v)| v)
    }

    /// Return an iterator over all edges in the structural graph.
    pub fn edges(&self) -> impl Iterator<Item = &Edge> {
        self.edges.iter()
    }

    /// Return a structural graph node by key.
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

    /// Returns the edge of an interface, if it is connected.
    pub fn get_edge(&self, io: NodeIORef) -> Option<&Edge> {
        self.edges.iter().find(|e| e.sink == io || e.source == io)
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
                        println!("        Nodes:");

                        // Check for failure of non-existent node.
                        assert!(structural.node(NodeKey::try_new("asdf").unwrap()).is_err());

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
