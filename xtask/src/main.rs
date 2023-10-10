use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::{bench::BenchCmd, clean::CleanCmd, es::EsCmd, git::GitCmd};

mod bench;
mod clean;
mod es;
mod git;
mod util;

#[derive(Debug, Parser)]
struct CliArgs {
    #[clap(subcommand)]
    cmd: Cmd,
}

#[derive(Debug, Subcommand)]
enum Cmd {
    Es(EsCmd),
    Bench(BenchCmd),
    Git(GitCmd),
    Clean(CleanCmd),
}

fn main() -> Result<()> {
    let args = CliArgs::parse();

    match args.cmd {
        Cmd::Es(c) => c.run(),
        Cmd::Bench(c) => c.run(),
        Cmd::Git(c) => c.run(),
        Cmd::Clean(c) => c.run(),
    }
}
