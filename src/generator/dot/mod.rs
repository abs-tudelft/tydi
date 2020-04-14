use crate::cat;
use crate::design::implementation::structural::THIS_KEY;
use crate::design::implementation::structural::{Edge, Node, StreamletInst, StructuralImpl};
use crate::design::implementation::Implementation;
use crate::design::{Interface, Library, Mode, Project, Streamlet};
use crate::generator::GenerateProject;
use crate::{Identify, Result};
use std::ops::Deref;
use std::path::Path;

// To be added later for Dot configuration from CLI:
// #[cfg(feature = "cli")]
// use structopt::StructOpt;

pub struct Colors([(&'static str, &'static str); 10]);

impl Default for Colors {
    fn default() -> Self {
        Colors([
            ("#ffffff", "#c4c4c4"), // 0 white
            ("#c4c4c4", "#808080"), // 1 gray
            ("#d65f5f", "#8c0800"), // 2 red
            ("#ee854a", "#b1400d"), // 3 orange
            ("#d5bb67", "#b8850a"), // 4 yellow
            ("#6acc64", "#12711c"), // 5 green
            ("#82c6e2", "#006374"), // 6 cyan
            ("#4878d0", "#001c7f"), // 7 blue
            ("#956cb4", "#591e71"), // 8 purple
            ("#dc7ec0", "#a23582"), // 9 pink
        ])
    }
}

pub struct DotConfig {
    colors: Colors,
}

impl DotConfig {
    pub fn node_color(&self, i: usize) -> String {
        format!(
            "fillcolor=\"{}\", color=\"{}\"",
            self.colors.0[i].0, self.colors.0[i].1
        )
    }

    pub fn cluster_border(&self, i: usize) -> String {
        format!("color=\"{}\";", self.colors.0[i].1)
    }

    pub fn cluster_fill(&self, i: usize) -> String {
        format!("color=\"{}\";", self.colors.0[i].0)
    }

    pub fn cluster_bgcolor(&self, i: usize) -> String {
        format!("bgcolor=\"{}\";", self.colors.0[i].0)
    }

    pub fn interface(&self, mode: Mode) -> String {
        format!(
            "{}, {}, {}, {}",
            "style=\"filled\"",
            "shape=ellipse",
            "penwidth=1",
            match mode {
                Mode::In => self.node_color(5),
                Mode::Out => self.node_color(6),
            },
        )
    }
}

impl Default for DotConfig {
    fn default() -> Self {
        DotConfig {
            colors: Colors::default(),
        }
    }
}

fn t(i: usize) -> String {
    " ".repeat(2 * i)
}

pub trait Dot {
    fn to_dot(&self, config: &DotConfig, indent: usize, prefix: &str) -> String;
}
impl<'p> Dot for Interface<'p> {
    fn to_dot(&self, config: &DotConfig, i: usize, prefix: &str) -> String {
        format!(
            "{}{} [label=\"{}\", {}];",
            t(i),
            vec![prefix, self.identifier().clone()].join("_").as_str(),
            self.identifier().clone(),
            config.interface(self.mode()),
        )
    }
}

fn io_subgraph<'p>(
    config: &DotConfig,
    i: usize,
    prefix: &str,
    interfaces: impl Iterator<Item = &'p Interface<'p>>,
    mode: Mode,
) -> String {
    format!(
        "{}subgraph cluster_{}_{} {{\n{}\n{}}}\n",
        t(i + 1),
        prefix,
        mode,
        format!(
            "{}{}{}",
            format!("{}label=\"\";\n", t(i + 2)),
            format!("{}style=invis;\n", t(i + 2)),
            interfaces
                .filter(|io| io.mode() == mode)
                .map(|io| io.to_dot(config, i + 2, prefix))
                .collect::<Vec<String>>()
                .join("\n")
        ),
        t(i + 1),
    )
}

fn cluster_style(config: &DotConfig, fill: usize, border: usize, i: usize) -> String {
    format!(
        "{}{}{}{}",
        format!("{}style=\"rounded\";\n", t(i + 1)),
        format!("{}{}\n", t(i + 1), config.cluster_bgcolor(fill)),
        format!("{}{}\n", t(i + 1), config.cluster_border(border)),
        format!("{}{}\n", t(i + 1), "penwidth=3;")
    )
}

impl<'p> Dot for Streamlet<'p> {
    fn to_dot(&self, config: &DotConfig, i: usize, prefix: &str) -> String {
        todo!()
        // let p = cat!(prefix, self.key());
        // let in_graph = io_subgraph(config, i, p.as_str(), self.interfaces(), Mode::In);
        // let out_graph = io_subgraph(config, i, p.as_str(), self.interfaces(), Mode::Out);
        // format!(
        //     "{}subgraph cluster_{} {{\n{}\n{}}}",
        //     t(i),
        //     p,
        //     format!(
        //         "{}{}{}{}{}",
        //         // label
        //         format!("{}label=\"{}\";\n", t(i + 1), self.key()),
        //         // style
        //         cluster_style(config, 1, 1, i + 1),
        //         // inputs
        //         in_graph,
        //         // outputs
        //         out_graph,
        //         // implementation
        //         self.implementation().to_dot(config, i + 1, p.as_str())
        //     ),
        //     t(i)
        // )
    }
}

impl<'p> Dot for Implementation {
    fn to_dot(&self, config: &DotConfig, i: usize, prefix: &str) -> String {
        match self {
            Implementation::Structural(s) => s.to_dot(config, i, prefix),
            _ => String::new(),
        }
    }
}

impl<'p> Dot for StreamletInst {
    fn to_dot(&self, config: &DotConfig, i: usize, prefix: &str) -> String {
        let p = cat!(prefix, self.key());
        todo!()
        // let ins = project
        //     .get_streamlet(self.streamlet())
        //     .unwrap()
        //     .interfaces();
        // let outs = project
        //     .get_streamlet(self.streamlet())
        //     .unwrap()
        //     .interfaces();
        // format!(
        //     "{}subgraph cluster_{} {{\n{}{}}}",
        //     t(i),
        //     p,
        //     format!(
        //         "{}{}{}{}",
        //         format!("{}label=\"{}\";\n", t(i + 1), self.key()),
        //         // style
        //         cluster_style(config, 4, 4, i + 1),
        //         // inputs
        //         io_subgraph(config, i, p.as_str(), ins, Mode::In),
        //         // outputs
        //         io_subgraph(config, i, p.as_str(), outs, Mode::Out),
        //     ),
        //     t(i)
        // )
    }
}

impl<'p> Dot for Node {
    fn to_dot(&self, config: &DotConfig, i: usize, prefix: &str) -> String {
        match self {
            Node::This(s) => s.to_dot(config, i, prefix),
            Node::Streamlet(s) => s.to_dot(config, i, prefix),
        }
    }
}

impl<'p> Dot for Edge {
    fn to_dot(&self, _: &DotConfig, i: usize, prefix: &str) -> String {
        let src = match self.source().node().deref() {
            THIS_KEY => cat!(prefix, self.source().interface()),
            _ => cat!(
                prefix,
                "impl",
                self.source().node(),
                self.source().interface()
            ),
        };
        let snk = match self.sink().node().deref() {
            THIS_KEY => cat!(prefix, self.sink().interface()),
            _ => cat!(prefix, "impl", self.sink().node(), self.sink().interface()),
        };
        format!("{}{} -> {};", t(i), src, snk)
    }
}

impl<'p> Dot for StructuralImpl {
    fn to_dot(&self, config: &DotConfig, i: usize, prefix: &str) -> String {
        let p = vec![prefix, "impl"].join("_");
        format!(
            "{}subgraph cluster_{} {{\n{}\n{}}}",
            t(i),
            p,
            format!(
                "{}{}{}\n{}",
                format!("{}label=\"Implementation\";\n", t(i + 1)),
                // style
                cluster_style(config, 0, 1, i + 1),
                // nodes
                self.nodes()
                    .filter(|n| n.key().deref() != THIS_KEY)
                    .map(|n| n.to_dot(config, i + 1, p.as_str()))
                    .collect::<Vec<String>>()
                    .join("\n"),
                // edges
                self.edges()
                    .map(|e| e.to_dot(config, i + 1, prefix))
                    .collect::<Vec<String>>()
                    .join("\n"),
            ),
            t(i)
        )
    }
}

impl<'p> Dot for Library<'p> {
    fn to_dot(&self, config: &DotConfig, i: usize, prefix: &str) -> String {
        format!(
            "digraph {{\n{}\n{}}}",
            format!(
                "{}{}{}",
                format!("{}rankdir=LR;\n", t(i + 1)),
                format!("{}splines=compound;\n", t(i + 1)),
                self.streamlets()
                    .map(|s| s.to_dot(
                        config,
                        i + 1,
                        vec![prefix, self.identifier()].join("_").as_str()
                    ))
                    .collect::<Vec<String>>()
                    .join("\n"),
            ),
            t(i)
        )
    }
}

/// A configurable VHDL back-end entry point.
#[derive(Default)]
pub struct DotBackend {}

impl GenerateProject for DotBackend {
    fn generate(&self, project: &Project, path: impl AsRef<Path>) -> Result<()> {
        // Create the project directory.
        let dir = path.as_ref().to_path_buf();

        for lib in project.libraries() {
            // Create sub-directory for each lib
            let mut lib_dir = dir.clone();
            lib_dir.push(project.identifier());
            std::fs::create_dir_all(lib_dir.as_path())?;

            // Determine output file
            let mut lib_path = lib_dir.clone();
            lib_path.push(lib.identifier());
            lib_path.set_extension("dot");

            let dot = lib.to_dot(&DotConfig::default(), 0, "");

            // TODO: remove this
            println!("{}", dot);

            std::fs::write(lib_path, dot)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::design::implementation::structural::builder::tests::builder_example;

    #[test]
    fn dot() {
        let tmpdir = tempfile::tempdir().unwrap();

        let prj = crate::design::project::tests::proj::single_lib_proj("test");
        let dot = DotBackend {};
        // TODO: implement actual test.

        assert!(dot.generate(&prj, tmpdir).is_ok());
    }

    #[test]
    fn dot_impl() {
        let tmpdir = tempfile::tempdir().unwrap();

        let prj = builder_example().unwrap();
        let dot = DotBackend {};
        // TODO: implement actual test.

        assert!(dot.generate(&prj, tmpdir).is_ok());
    }
}
