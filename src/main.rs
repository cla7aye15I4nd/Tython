use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use tython::compiler::Compiler;

#[derive(Parser, Debug)]
#[command(name = "tython")]
#[command(about = "A Python to native code compiler", long_about = None)]
struct Args {
    #[arg(value_name = "FILE")]
    input: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut compiler = Compiler::new(args.input.clone())?;

    let exe_path = args
        .input
        .canonicalize()
        .unwrap()
        .with_extension(std::env::consts::EXE_EXTENSION);
    compiler.compile(exe_path.clone())?;

    let output = std::process::Command::new(&exe_path).output()?;

    std::io::Write::write_all(&mut std::io::stdout(), &output.stdout)?;
    std::io::Write::write_all(&mut std::io::stderr(), &output.stderr)?;

    std::process::exit(output.status.code().unwrap_or(1));
}
