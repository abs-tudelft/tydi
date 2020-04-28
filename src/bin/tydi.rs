//! Tydi generator Command-Line Interface
//!
//! The Command-Line Interface binary is enabled by the `cli` feature flag.

use log::{debug, info, LevelFilter};
use std::convert::TryInto;
use std::path::{Path, PathBuf};
use tydi::generator::vhdl::{VHDLBackEnd, VHDLConfig};
use tydi::generator::GenerateProject;
use tydi::UniqueKeyBuilder;
use tydi::{Logger, Result};

use structopt::StructOpt;
use tydi::design::{Library, Project};
use tydi::generator::dot::DotBackend;

static LOGGER: Logger = Logger;

/// Back-end options.
#[derive(Debug, StructOpt)]
enum TargetOpt {
    /// Generate VHDL sources.
    VHDL(VHDLConfig),
    /// Generate Chisel sources.
    Chisel,
    /// Gerate DOT graphs.
    Dot,
}

#[derive(Debug, StructOpt)]
struct GenerateOpts {
    /// Name of the project to generate.
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

    #[structopt(subcommand)]
    target: TargetOpt,
}

/// Top-level CLI commands
#[derive(Debug, StructOpt)]
enum Command {
    /// Generate HDL output from Streamlet Definition Files.
    Generate(GenerateOpts),
}

#[derive(Debug, StructOpt)]
pub struct Opt {
    /// Enable verbose logging.
    #[structopt(short, long)]
    verbose: bool,
    /// Enable debug-level logging.
    #[structopt(short, long)]
    debug: bool,
    #[structopt(subcommand)]
    cmd: Command,
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
    let mut lib_builder = UniqueKeyBuilder::new();
    for i in input_files {
        lib_builder.add_item(Library::from_file(i.as_path())?);
    }

    // Construct the project from the libraries.
    let project = Project::from_builder(opts.name.try_into()?, lib_builder)?;

    info!("Generating sources...");
    match opts.target {
        TargetOpt::VHDL(cfg) => {
            let vhdl: VHDLBackEnd = cfg.into();
            vhdl.generate(
                &project,
                opts.output.unwrap_or(std::env::current_dir()?).as_path(),
            )?;
        }
        TargetOpt::Chisel => unimplemented!(),
        TargetOpt::Dot => {
            let dot = DotBackend::default();
            dot.generate(
                &project,
                opts.output.unwrap_or(std::env::current_dir()?).as_path(),
            )?;
        }
    }
    info!("Done.");
    Ok(())
}

/// Internal main function wrapped with CLI main function.
/// Useful for tests.
pub fn internal_main(options: Opt) -> Result<()> {
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

/// CLI main function.
fn main() -> Result<()> {
    // Set up logger.
    log::set_logger(&LOGGER)?;
    // Run Tydi
    internal_main(Opt::from_args())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_vhdl() -> Result<()> {
        log::set_logger(&LOGGER)?;
        let tmpdir = tempfile::TempDir::new()?;
        let sdf_file = tmpdir.path().join("lib.sdf");
        std::fs::write(
            sdf_file.as_path(),
            "Streamlet x ( a : in Stream<Bits<1>, d=1>, b : out Stream<Bits<32>> )",
        )?;
        internal_main(
            Opt::from_iter_safe(vec![
                "tydi",
                "--debug",
                "generate",
                format!("-i{}", sdf_file.to_str().unwrap()).as_str(),
                format!("-o{}", tmpdir.path().to_str().unwrap()).as_str(),
                "test",
                "vhdl",
                "-a=fancy",
                "-s=gen",
            ])
            .map_err(|e| panic!(format!("{}", e)))
            .unwrap(),
        )?;
        let expected_vhdl = tmpdir.path().join("test/lib_pkg.gen.vhd");
        std::fs::metadata(expected_vhdl)?;
        std::fs::remove_dir_all(tmpdir.path())?;

        // TODO: for some reason these two functions cannot be tested seperately in cargo

        //     Ok(())
        // }
        //
        // #[test]
        // fn cli_dot() -> Result<()> {
        //     log::set_logger(&LOGGER)?;
        let tmpdir = tempfile::TempDir::new()?;
        let sdf_file = tmpdir.path().join("lib.sdf");
        std::fs::write(
            sdf_file.as_path(),
            "Streamlet x ( a : in Stream<Bits<1>, d=1>, b : out Stream<Bits<32>> )",
        )?;
        internal_main(
            Opt::from_iter_safe(vec![
                "tydi",
                "--debug",
                "generate",
                format!("-i{}", sdf_file.to_str().unwrap()).as_str(),
                format!("-o{}", tmpdir.path().to_str().unwrap()).as_str(),
                "test",
                "dot",
            ])
            .map_err(|e| panic!(format!("{}", e)))
            .unwrap(),
        )?;
        let expected_dot = tmpdir.path().join("test/lib.dot");
        std::fs::metadata(expected_dot)?;
        //std::fs::remove_dir_all(tmpdir.path())?;
        Ok(())
    }
}
