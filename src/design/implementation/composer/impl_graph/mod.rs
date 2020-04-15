use std::cell::{Ref, RefMut};
use std::collections::HashMap;
use std::convert::TryInto;
use std::fmt::Debug;
use std::rc::Rc;

use nom::lib::std::fmt::Formatter;

use crate::design::implementation::composer::GenericComponent;
use crate::design::{IFKey, Interface, NodeIFHandle, NodeKey, StreamletHandle, StreamletKey};
use crate::{Error, Result};



#[derive(Clone, Debug, PartialEq)]
pub struct Edge {
    pub(crate) source: NodeIFHandle,
    pub(crate) sink: NodeIFHandle,
}

impl Edge {
    pub fn source(&self) -> NodeIFHandle {
        self.source.clone()
    }
    pub fn sink(&self) -> NodeIFHandle {
        self.sink.clone()
    }
}

#[derive(Clone)]
pub struct Node {
    pub(crate) key: NodeKey,
    pub(crate) item: Rc<dyn GenericComponent>,
}

impl Node {
    pub fn key(&self) -> NodeKey {
        self.key.clone()
    }

    pub fn iface(&self, key: IFKey) -> Result<Ref<Interface>> {
        /*match self.key().deref() {
            THIS_KEY => {
                self.item.get_interface(key).map(|i| i.reverse())
            },
            _ => self.item.get_interface(key),
        }*/
        self.item.get_interface(key)
    }

    pub fn iface_mut(&self, key: IFKey) -> Result<RefMut<Interface>> {
        /*match self.key() {
            THIS_KEY => {
                self.item.get_interface(key).map(|i| i.reversed())
            },
            _ => self.item.get_interface(key),
        }*/
        self.item.get_interface_mut(key)
    }

    pub fn io<K>(&self, key: K) -> Result<NodeIFHandle>
    where
        K: TryInto<IFKey>,
        <K as TryInto<IFKey>>::Error: Into<Error>,
    {
        let key = key.try_into().map_err(Into::into)?;
        Ok(NodeIFHandle {
            node: self.key(),
            iface: key,
        })
    }

    pub fn this(&self) -> NodeKey {
        self.key.clone()
    }

    pub fn component(&self) -> Rc<dyn GenericComponent> {
        self.item.clone()
    }
}

#[derive(Clone)]
pub struct ImplementationGraph {
    pub(crate) streamlet: StreamletHandle,
    pub(crate) edges: Vec<Edge>,
    pub(crate) nodes: HashMap<NodeKey, Node>,
}

impl ImplementationGraph {
    pub fn streamlet_key(&self) -> StreamletKey {
        self.streamlet.streamlet().clone()
    }
    pub fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.nodes.iter().map(|(_, i)| i)
    }
    pub fn edges(&self) -> impl Iterator<Item = &Edge> {
        self.edges.iter()
    }
    pub fn get_node(&self, key: NodeKey) -> Result<&Node> {
        let node = self.nodes.get(&key);
        match node {
            Some(n) => Ok(n),
            None => Err(Error::ComposerError(format!(
                "Error while retrieving node {:?}, it does not exist in design.",
                key
            ))),
        }
    }
    pub fn get_edge(&self, iface: NodeIFHandle) -> Result<&Edge> {
        let edge = self
            .edges
            .iter()
            .find(|e| e.sink == iface || e.source == iface);
        match edge {
            Some(e) => Ok(e),
            None => Err(Error::ComposerError(format!(
                "Error while retrieving connection for interface {:?}, it does not exist in design.",
                iface
            )))
        }
    }
    pub fn this(&self) -> &Node {
        self.nodes.get(&NodeKey::this()).unwrap()
    }
    pub fn streamlet(self) -> StreamletHandle {
        self.streamlet.clone()
    }
}

impl Debug for ImplementationGraph {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        unimplemented!()
    }
}

impl PartialEq for ImplementationGraph {
    fn eq(&self, other: &ImplementationGraph) -> bool {
        self.streamlet.streamlet() == other.streamlet_key()
    }
}
