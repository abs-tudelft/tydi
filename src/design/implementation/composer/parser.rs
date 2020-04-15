use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::ops::Deref;
use std::rc::Rc;

use pest::{Parser, RuleType};
use pest::iterators::Pair;

use crate::{Error, Name, Result, Reversed, UniqueKeyBuilder};
use crate::design::{
    GEN_LIB, IFKey, LibKey, Library, Mode, NodeIFHandle, NodeKey, Project, Streamlet,
    StreamletHandle, StreamletKey,
};
use crate::design::implementation::composer::impl_graph::{Edge, ImplementationGraph, Node};
use crate::design::implementation::composer::GenericComponent;
use crate::design::implementation::composer::patterns::{FilterStream, MapStream, ReduceStream};
use crate::design::implementation::Implementation;
use crate::design::implementation::Implementation::Structural;
use crate::error::LineErr;

#[derive(Parser)]
#[grammar = "design/implementation/composer/impl.pest"]
pub struct ImplDef;

pub trait LineNum {
    fn line_num(&self) -> usize;
}
impl<'i, R: RuleType> LineNum for Pair<'i, R> {
    fn line_num(&self) -> usize {
        self.as_span().start_pos().line_col().0
    }
}

fn match_rule<T>(
    pair: Pair<Rule>,
    rule: Rule,
    mut f: impl FnMut(Pair<Rule>) -> Result<T>,
) -> Result<T> {
    if pair.as_rule() == rule {
        f(pair)
    } else {
        Err(Error::ImplParsingError(LineErr::new(
            pair.line_num(),
            format!("Expected: \"{:?}\", Actual: \"{:?}\"", rule, pair),
        )))
    }
}

pub struct ImplParser<'i> {
    project: &'i mut Project,
    body: Pair<'i, Rule>,
    imp: Implementation,
}

impl<'i> ImplParser<'i> {
    pub fn try_new(project: &'i mut Project, input: &'i str) -> Result<Self> {
        let pair = ImplDef::parse(Rule::implementation, input)
            .map_err(|e| {
                Error::ImplParsingError(LineErr::new(
                    0,
                    format!("Implementation parsing error: {}", e),
                ))
            })?
            .next()
            .unwrap();

        let mut pairs = pair.into_inner();
        let streamlet_handle: StreamletHandle = pairs.next().unwrap().try_into()?;

        //let pair = pairs.next().unwrap();

        let s = project
            .get_lib(streamlet_handle.lib())?
            .get_streamlet(streamlet_handle.streamlet())?
            .clone();

        let gen_lib = Library::new(LibKey::try_new(GEN_LIB).unwrap());
        project.add_lib(gen_lib)?;

        //Create a streamlet with reversed interfaces
        let this_streamlet = Streamlet::from_builder(
            StreamletKey::try_from(s.key()).unwrap(),
            UniqueKeyBuilder::new().with_items(s.interfaces().map(|i| i.deref().reversed())),
            None,
        )?;

        Ok(ImplParser {
            project,
            //Safe to unwrap, Pest guarantees that there's an implementation body.
            body: pairs.next().unwrap(),
            imp: Implementation::Structural(ImplementationGraph {
                streamlet: streamlet_handle,
                edges: vec![],
                nodes: vec![(
                    NodeKey::this(),
                    Node {
                        key: NodeKey::this(),
                        item: Rc::new(this_streamlet.clone()),
                    },
                )]
                .into_iter()
                .collect::<HashMap<NodeKey, Node>>(),
            }),
        })
    }

    pub fn transform_body(&mut self) -> Result<()> {
        match &mut self.body.as_rule() {
            Rule::structural => self.transform_structural(),
            _ => unimplemented!(),
        }
    }

    pub fn transform_structural(&mut self) -> Result<()> {
        //Step to structural_body
        let pair = self.body.clone().into_inner().next().unwrap();
        //structural_body inner
        //let pair = pair.into_inner().next().unwrap();
        for pair in pair.into_inner() {
            match &pair.as_rule() {
                Rule::node => {
                    let node_tuple = self.transform_node(pair)?;
                    let node = node_tuple.1;
                    self.insert_node(node)?;
                    for edges in node_tuple.3 {
                        self.connect(edges)?;
                    }
                }
                Rule::connection => {
                    let edge = Edge::try_from(pair)?;
                    self.connect(edge)?
                }
                Rule::chain_connection => {
                    self.transform_chain_connection(pair)?;
                }
                _ => unimplemented!(),
            }
        }
        Ok(())
    }

    pub fn transform_node(
        &mut self,
        pair: Pair<Rule>,
    ) -> Result<(Name, Node, StreamletHandle, Vec<Edge>)> {
        //{ ident ~ ":" ~  (pattern | streamlet_inst) }
        let mut pairs = pair.into_inner();
        //ident
        let name_pair = pairs.next().unwrap();
        let key = Name::try_from(name_pair).unwrap();
        //(pattern | streamlet_inst)
        let pair = pairs.next().unwrap();
        match pair.as_rule() {
            Rule::streamlet_inst => {
                let node_tuple = self.transform_streamlet_inst(pair, key.clone())?;
                let node = Node {
                    key: key.clone(),
                    item: node_tuple.0,
                };
                Ok((key.clone(), node, node_tuple.1, Vec::new()))
            }
            Rule::pattern => {
                let node_tuple = self.transform_pattern(pair, key.clone())?;
                let node = Node {
                    key: key.clone(),
                    item: node_tuple.0,
                };
                Ok((key.clone(), node, node_tuple.1, node_tuple.2))
            }
            _ => unreachable!(),
        }
    }

    pub fn transform_streamlet_inst(
        &mut self,
        pair: Pair<Rule>,
        _key: Name,
    ) -> Result<(Rc<dyn GenericComponent>, StreamletHandle, Vec<Edge>)> {
        //{ streamlet_handle ~ ("[" ~ (parameter_assign)+ ~ "]")? }
        let mut pairs = pair.into_inner();

        //streamlet_handle
        let streamlet_handle_pair = pairs.next().unwrap();
        let streamlet_handle = StreamletHandle::try_from(streamlet_handle_pair)?;
        let streamlet = self
            .project
            .get_lib(streamlet_handle.lib())?
            .get_streamlet(streamlet_handle.streamlet())?
            .clone();
        Ok((Rc::new(streamlet.clone()), streamlet_handle, Vec::new()))
    }

    pub fn transform_pattern(
        &mut self,
        pair: Pair<Rule>,
        key: Name,
    ) -> Result<(Rc<dyn GenericComponent>, StreamletHandle, Vec<Edge>)> {
        //{ map_stream | filter_stream | reduce_stream }
        let pair = pair.into_inner().next().unwrap();
        match pair.as_rule() {
            Rule::map_stream => self.transform_map_stream(pair, key),
            Rule::reduce_stream => self.transform_reduce_stream(pair, key),
            Rule::filter_stream => self.transform_filter_stream(pair, key),
            _ => unreachable!(),
        }
    }

    pub fn transform_map_stream(
        &mut self,
        pair: Pair<Rule>,
        key: Name,
    ) -> Result<(Rc<dyn GenericComponent>, StreamletHandle, Vec<Edge>)> {
        let op = self.transform_node(pair.into_inner().next().unwrap())?;

        let name = Name::try_from(format!("{}_gen", key.to_string()))?;
        let mut component = MapStream::try_new(self.project, name.clone(), op.2)?;
        component.with_backend(
            name.clone(),
            StreamletHandle {
                lib: Name::try_new(GEN_LIB)?,
                streamlet: name.clone(),
            },
        )?;
        let object = component.finish();
        let handle = self
            .project
            .get_lib_mut(Name::try_from(GEN_LIB).unwrap())?
            .add_streamlet(object.streamlet().clone())?;
        Ok((Rc::new(object), handle, Vec::new()))
    }

    pub fn transform_reduce_stream(
        &mut self,
        pair: Pair<Rule>,
        key: Name,
    ) -> Result<(Rc<dyn GenericComponent>, StreamletHandle, Vec<Edge>)> {
        let op = self.transform_node(pair.into_inner().next().unwrap())?;

        let name = Name::try_from(format!("{}_gen", key.to_string()))?;
        let mut component = ReduceStream::try_new(self.project, name.clone(), op.2)?;
        component.with_backend(
            name.clone(),
            StreamletHandle {
                lib: Name::try_new(GEN_LIB)?,
                streamlet: name.clone(),
            },
        )?;
        let object = component.finish();
        let handle = self
            .project
            .get_lib_mut(Name::try_from(GEN_LIB).unwrap())?
            .add_streamlet(object.streamlet().clone())?;
        Ok((Rc::new(object), handle, Vec::new()))
    }

    pub fn transform_filter_stream(
        &mut self,
        pair: Pair<Rule>,
        key: Name,
    ) -> Result<(Rc<dyn GenericComponent>, StreamletHandle, Vec<Edge>)> {
        let predicate = NodeIFHandle::try_from(pair.into_inner().next().unwrap())?;

        let name = Name::try_from(format!("{}_gen", key.to_string()))?;
        let mut component = FilterStream::try_new(self.project, name.clone())?;
        component.with_backend(
            name.clone(),
            StreamletHandle {
                lib: Name::try_new(GEN_LIB)?,
                streamlet: name.clone(),
            },
        )?;
        let object = component.finish();
        let handle = self
            .project
            .get_lib_mut(Name::try_from(GEN_LIB).unwrap())?
            .add_streamlet(object.streamlet().clone())?;
        let edges = vec![Edge {
            source: predicate.clone(),
            sink: NodeIFHandle {
                node: key.clone(),
                iface: IFKey::try_new("pred")?,
            },
        }];
        Ok((Rc::new(object), handle, edges))
    }

    pub fn transform_chain_connection(&mut self, pair: Pair<Rule>) -> Result<()> {
        //{ (ident | node_if_handle_list) ~ "<=>" ~ (ident | node_if_handle_list) }
        let mut pairs = pair.into_inner();

        //(ident | node_if_handle_list)
        let pair = pairs.next().unwrap();

        let mut src = match pair.as_rule() {
            Rule::ident => Name::try_from(pair.clone()),
            _ => unimplemented!(),
        }?;

        for pair in pairs {
            let dst = match pair.as_rule() {
                Rule::ident => Name::try_from(pair),
                _ => unimplemented!(),
            }?;

            let src_i = match &mut self.imp {
                Structural(ref mut s) => {
                    match s
                        .get_node(src.clone())?
                        .component()
                        .outputs()
                        .find(|i| i.key().to_string() == "out".to_string())
                    {
                        Some(i) => Ok(i.clone()),
                        None => Err(Error::ComposerError(format!(
                            "Chain connection left side doesn't have an output interface: {:?}",
                            src
                        ))),
                    }
                }
                _ => unreachable!(),
            }?;

            let dst_i = match &mut self.imp {
                Structural(ref mut s) => {
                    match s.get_node(dst.clone())?.component().inputs().find(|i| i.key().to_string() == "in".to_string()) {
                        Some(i) => Ok(i.clone()),
                        None => Err(Error::ComposerError(format!(
                            "Chain connection right side doesn't have a matching input interface: {:?}",
                            dst
                        ))),
                    }
                }
                _ => unreachable!(),
            }?;

            let edge = Edge {
                source: NodeIFHandle {
                    node: src.clone(),
                    iface: src_i.key().clone(),
                },
                sink: NodeIFHandle {
                    node: dst.clone(),
                    iface: dst_i.key().clone(),
                },
            };
            self.connect(edge)?;
            src = dst;
        }
        Ok(())
    }

    pub fn connect(&mut self, edge: Edge) -> Result<()> {
        match &mut self.imp {
            Structural(ref mut s) => {
                //Deal with type inferences
                let src_if = s
                    .get_node(edge.clone().source().node)?
                    .iface(edge.clone().source().iface)?
                    .deref()
                    .clone();
                let dst_if = s
                    .get_node(edge.clone().sink().node)?
                    .iface(edge.clone().sink().iface)?
                    .deref()
                    .clone();
                let src_type = src_if.typ().clone();
                let dst_type = dst_if.typ().clone();

                s.get_node(edge.clone().source().node)?
                    .iface_mut(edge.clone().source().iface)?
                    .infer_type(dst_type.clone())?;
                s.get_node(edge.clone().sink().node)?
                    .iface_mut(edge.clone().sink().iface)?
                    .infer_type(src_type.clone())?;

                //Run action handlers for connection
                s.get_node(edge.clone().source().node)?
                    .component()
                    .connect_action()?;
                s.get_node(edge.clone().sink().node)?
                    .component()
                    .connect_action()?;

                //Refresh the data types afgter propagation for checks
                let src_if = s
                    .get_node(edge.clone().source().node)?
                    .iface(edge.clone().source().iface)?
                    .deref()
                    .clone();
                let dst_if = s
                    .get_node(edge.clone().sink().node)?
                    .iface(edge.clone().sink().iface)?
                    .deref()
                    .clone();

                if src_if.mode() != Mode::Out {
                    Err(Error::ComposerError(format!(
                        "Interface {:?} is not an output.",
                        edge.clone().source()
                    )))
                } else if dst_if.mode() != Mode::In {
                    Err(Error::ComposerError(format!(
                        "Interface {:?} is not an input.",
                        edge.clone().source()
                    )))
                } else if s.get_edge(edge.clone().source()).is_ok() {
                    Err(Error::ComposerError(format!(
                        "Cannot connect {:?} to {:?}, source is already connected.",
                        edge.clone().sink(),
                        edge.clone().source()
                    )))
                } else if s.get_edge(edge.clone().sink()).is_ok() {
                    Err(Error::ComposerError(format!(
                        "Cannot connect {:?} to {:?}, sink is already connected.",
                        edge.clone().sink(),
                        edge.clone().source()
                    )))
                } else if src_if.typ() != dst_if.typ() {
                    Err(Error::ComposerError(format!(
                        "Type incompatibility between sink {:?} : {:?}, and source {:?} : {:?}.",
                        edge.clone().sink(),
                        dst_type,
                        edge.clone().source(),
                        src_type
                    )))
                } else {
                    s.edges.push(edge);
                    Ok(())
                }?;
            }
            _ => unreachable!(),
        }
        Ok(())
    }

    pub fn insert_node(&mut self, node: Node) -> Result<()> {
        match &mut self.imp {
            Structural(ref mut s) => match s.nodes.insert(node.clone().key(), node.clone()) {
                None => Ok(()),
                Some(_lib) => Err(Error::ComposerError(format!(
                    "Instance {} already exists.",
                    node.key()
                ))),
            },
            _ => unreachable!(),
        }
    }

    pub fn this(&self) -> Node {
        match &self.imp {
            Structural(s) => {
                // We can unwrap safely here because the "this" node should always exist.
                s.nodes.get(&NodeKey::this()).unwrap().clone()
            }
            _ => unreachable!(),
        }
    }

    pub fn finish(self) -> Implementation {
        self.imp
    }
}

impl<'i> TryFrom<Pair<'i, Rule>> for StreamletHandle {
    type Error = Error;
    fn try_from(pair: Pair<Rule>) -> Result<Self> {
        match_rule(pair, Rule::streamlet_handle, |pair| {
            let mut pairs = pair.into_inner();
            let lib = pairs.next().unwrap().as_str();
            let streamlet = pairs.next().unwrap().as_str();
            let lib_key = LibKey::try_from(lib)?;
            let streamlet_key = StreamletKey::try_from(streamlet)?;
            Ok(StreamletHandle {
                lib: lib_key,
                streamlet: streamlet_key,
            })
        })
    }
}

impl<'i> TryFrom<Pair<'i, Rule>> for Name {
    type Error = Error;
    fn try_from(pair: Pair<Rule>) -> Result<Self> {
        match_rule(pair.clone(), Rule::ident, |pair| {
            Name::try_from(pair.clone().as_str())
        })
    }
}

impl<'i> TryFrom<Pair<'i, Rule>> for NodeIFHandle {
    type Error = Error;
    fn try_from(pair: Pair<Rule>) -> Result<Self> {
        match_rule(pair, Rule::node_if_handle, |pair| {
            let mut pairs = pair.into_inner();
            let node = pairs.next().unwrap().as_str();
            let node = Name::try_from(node)?;

            let iface = pairs.next().unwrap().as_str();
            let iface = Name::try_from(iface)?;
            Ok(NodeIFHandle { node, iface })
        })
    }
}

impl<'i> TryFrom<Pair<'i, Rule>> for Edge {
    type Error = Error;
    fn try_from(pair: Pair<Rule>) -> Result<Self> {
        match_rule(pair, Rule::connection, |pair| {
            let mut pairs = pair.into_inner();

            let sink = NodeIFHandle::try_from(pairs.next().unwrap())?;
            let source = NodeIFHandle::try_from(pairs.next().unwrap())?;

            Ok(Edge { source, sink })
        })
    }
}

#[cfg(test)]
pub mod tests {
    use std::convert::TryFrom;

    use crate::{Name, Result};
    use crate::design::implementation::composer::tests::composition_test_proj;
    use crate::design::StreamletHandle;

    use super::*;

    pub fn impl_parser_test() -> Result<Project> {
        let mut prj = composition_test_proj()?;
        let top_impl = include_str!("../../../../tests/implementations/composition_example.impl");

        let mut builder = ImplParser::try_new(&mut prj, &top_impl)?;
        builder.transform_body().unwrap();
        let imp = builder.finish();
        prj.add_streamlet_impl(
            StreamletHandle {
                lib: Name::try_from("compositions")?,
                streamlet: Name::try_from("Top_level")?,
            },
            imp,
        )?;
        Ok(prj)
    }

    #[test]
    fn parser() -> Result<()> {
        let mut prj = composition_test_proj()?;

        let top_impl = include_str!("../../../../tests/implementations/composition_example.impl");

        let mut builder = ImplParser::try_new(&mut prj, &top_impl).unwrap();
        builder.transform_body().unwrap();
        let _imp = builder.finish();
        Ok(())
    }
}

