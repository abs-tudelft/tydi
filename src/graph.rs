use crate::{logical::LogicalStreamType, Result};
use indexmap::IndexMap;
use petgraph::prelude::*;
use std::collections::HashMap;

#[derive(Copy, Clone, Debug)]
enum Mode {
    In,
    Out,
}

#[derive(Debug)]
struct Interface {
    mode: Mode,
    logical_type: LogicalStreamType,
}

impl Interface {
    fn logical_type(&self) -> &LogicalStreamType {
        &self.logical_type
    }
}

#[derive(Debug)]
struct Streamlet {
    interfaces: IndexMap<String, Interface>,
}

impl Streamlet {
    fn get_interface(&self, key: &str) -> Option<&Interface> {
        self.interfaces.get(key)
    }
}

#[derive(Debug)]
struct Library {
    streamlet: HashMap<String, Streamlet>,
}

impl Library {
    fn get_streamlet(&self, key: &str) -> Option<&Streamlet> {
        self.streamlet.get(key)
    }
}

#[derive(Debug)]
struct Edge<'a> {
    source: (&'a Streamlet, &'a str),
    sink: (&'a Streamlet, &'a str),
}

#[derive(Debug)]
struct Node<'a> {
    name: String,
    streamlet: &'a Streamlet,
}

#[derive(Debug)]
struct Composer<'a> {
    library: &'a Library,
    instance_map: HashMap<String, NodeIndex>,
    graph: Graph<Node<'a>, Edge<'a>>,
}

impl<'a> Composer<'a> {
    fn get_streamlet(&self, name: &str) -> Result<&'a Streamlet> {
        Ok(self
            .library
            .get_streamlet(name)
            .ok_or_else(|| "library has no definition for this streamlet")?)
    }
    fn get_instance(&self, name: &str) -> Option<&Streamlet> {
        match self.get_instance_idx(name).ok() {
            Some(idx) => Some(self.graph[idx].streamlet),
            None => None,
        }
    }
    fn get_instance_idx(&self, name: &str) -> Result<NodeIndex> {
        Ok(self
            .instance_map
            .get(name)
            .copied()
            .ok_or_else(|| "graph has no instance with this name")?)
    }
    fn instantiate(&mut self, streamlet: &str, name: &str) -> Result<NodeIndex> {
        match self.instance_map.get(name) {
            Some(_) => Err("instance with this name already exists")?,
            None => {
                let idx = self.graph.add_node(Node {
                    name: name.into(),
                    streamlet: self.get_streamlet(streamlet)?,
                });
                self.instance_map.insert(name.to_string(), idx);
                Ok(idx)
            }
        }
    }
    fn connect(&mut self, source: (&str, &'a str), sink: (&str, &'a str)) -> Result<EdgeIndex> {
        let source_idx = self.get_instance_idx(source.0)?;
        let sink_idx = self.get_instance_idx(sink.0)?;
        Ok(self.graph.add_edge(
            source_idx,
            sink_idx,
            Edge {
                source: (self.graph[source_idx].streamlet, source.1),
                sink: (self.graph[sink_idx].streamlet, sink.1),
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() -> Result<()> {
        let library = Library {
            streamlet: vec![(
                "empty".to_string(),
                Streamlet {
                    interfaces: vec![
                        (
                            "in".to_string(),
                            Interface {
                                mode: Mode::In,
                                logical_type: LogicalStreamType::Null,
                            },
                        ),
                        (
                            "out".to_string(),
                            Interface {
                                mode: Mode::Out,
                                logical_type: LogicalStreamType::Null,
                            },
                        ),
                    ]
                    .into_iter()
                    .collect::<IndexMap<String, Interface>>(),
                },
            )]
            .into_iter()
            .collect::<HashMap<_, _>>(),
        };

        let mut composer = Composer {
            library: &library,
            instance_map: HashMap::new(),
            graph: Graph::new(),
        };

        composer.instantiate("empty", "first_empty")?;
        composer.instantiate("empty", "second_empty")?;
        composer.connect(("first_empty", "in"), ("second_empty", "out"))?;
        composer.connect(("second_empty", "in"), ("first_empty", "out"))?;

        assert_eq!(
            composer
                .get_instance("first_empty")
                .unwrap()
                .get_interface("in")
                .unwrap()
                .logical_type(),
            &LogicalStreamType::Null
        );
        dbg!(&composer);

        Ok(())
    }
}
