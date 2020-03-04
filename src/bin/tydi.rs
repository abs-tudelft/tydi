//! Tydi generator Command-Line Interface
//!
//! The Command-Line Interface binary is enabled by the `cli` feature flag.

use log::{debug, info, LevelFilter};
use std::convert::TryInto;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tydi::generator::vhdl::VHDLBackEnd;
use tydi::generator::{common, GenerateProject};
use tydi::UniquelyNamedBuilder;
use tydi::{Error, Logger, Result};

use structopt::StructOpt;
use tydi::design::{Library, Project};

static LOGGER: Logger = Logger;

/// Back-end options.
#[derive(Debug, StructOpt)]
enum TargetOpt {
    #[structopt(help = "Generate a VHDL project.")]
    VHDL,
    #[structopt(help = "Generate a Chisel project.")]
    Chisel,
}

impl FromStr for TargetOpt {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "vhdl" => Ok(TargetOpt::VHDL),
            "chisel" => Ok(TargetOpt::Chisel),
            _ => Err(Error::InvalidTarget(s.to_string())),
        }
    }
}

#[derive(Debug, StructOpt)]
struct GenerateOpts {
    #[structopt(help = "Output target. Possible values: vhdl, chisel")]
    target: TargetOpt,

    #[structopt(help = "Project name.")]
    name: String,

    #[structopt(
        short,
        help = "Streamlet Definition Files to generate output from.\n\
                If not supplied, all .sdf files in the current directory are used."
    )]
    inputs: Option<Vec<PathBuf>>,

    #[structopt(
        short,
        help = "Output directory for generated files.\n\
                If not supplied, the target name is used."
    )]
    output: Option<PathBuf>,
}

/// Top-level CLI commands
#[derive(Debug, StructOpt)]
#[structopt(about = "Tydi command-line interface.")]
enum Command {
    /// Generate HDL output from Streamlet Definition Files.
    Generate(GenerateOpts),
}

#[derive(Debug, StructOpt)]
pub struct Opt {
    #[structopt(subcommand)]
    cmd: Command,
    #[structopt(long = "log-verbose", help = "Enables informative logging.")]
    verbose: bool,
    #[structopt(long = "log-debug", help = "Enables debug-level logging.")]
    debug: bool,
}

/// Return all .sdf files in a path.
fn list_all_sdf(path: &Path) -> Result<Vec<PathBuf>> {
    let sdf_files: Vec<PathBuf> = std::fs::read_dir(path)?
        .filter_map(|e| e.ok())
        .map(|de| de.path())
        .filter(|p| p.extension().unwrap_or_default() == "sdf")
        .collect();
    Ok(sdf_files)
}

/// Generate sources from options.
fn generate(opts: GenerateOpts) -> Result<()> {
    info!("Loading Streamlet Definition Files...");
    // Obtain all input files from options.
    // If no option is given, get all .sdf files in the current path.
    let input_files = opts
        .inputs
        .unwrap_or(list_all_sdf(std::env::current_dir()?.as_path())?);

    let input_file_names: Vec<&str> = input_files.iter().filter_map(|pb| pb.to_str()).collect();
    debug!("Inputs: {}", input_file_names.join(", "));

    // Build up a set of uniquely named libraries.
    let mut lib_builder = UniquelyNamedBuilder::new();
    for i in input_files {
        lib_builder.add_item(Library::from_file(i.as_path())?);
    }

    // Construct the project from the libraries.
    let project = Project::from_builder(opts.name.try_into()?, lib_builder)?;

    // Convert the Tydi project to the common hardware representation.
    info!("Lowering Streamlet abstraction...");
    let common_project: common::Project = project.into();

    info!("Generating sources...");
    match opts.target {
        TargetOpt::VHDL => {
            let vhdl = VHDLBackEnd::default();
            vhdl.generate(
                &common_project,
                opts.output.unwrap_or(std::env::current_dir()?).as_path(),
            )?;
        }
        TargetOpt::Chisel => {}
    }
    info!("Done.");
    Ok(())
}

/// Internal main function wrapped with CLI main function.
/// Useful for tests.
pub fn internal_main(options: Opt) -> Result<()> {
    // Set up logger.
    log::set_logger(&LOGGER)?;
    if options.verbose {
        log::set_max_level(LevelFilter::Info);
    }
    if options.debug {
        log::set_max_level(LevelFilter::Debug);
        debug!("Debug-level logging enabled.");
    }

    match options.cmd {
        Command::Generate(gen_opts) => generate(gen_opts),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli() -> Result<()> {
        let tmpdir = tempfile::tempdir()?;
        std::env::set_current_dir(tmpdir.path())?;
        let sdf_file = tmpdir.path().join("test.sdf");
        std::fs::write(
            sdf_file.as_path(),
            "Streamlet x ( a : in Stream<Bits<1>, d=1>, b : out Stream<Bits<32>> )",
        )?;
        internal_main(Opt::from_iter(vec![
            "tydi",
            "--log-debug",
            "generate",
            "vhdl",
            "test",
        ]))?;
        let expected_vhdl = tmpdir.path().join("test/test_pkg.gen.vhd");
        std::fs::metadata(expected_vhdl)?;
        std::fs::remove_dir_all(tmpdir.path())?;
        Ok(())
    }
}

/// CLI main function.
fn main() -> Result<()> {
    internal_main(Opt::from_args())
}
