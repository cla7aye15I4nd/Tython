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

    #[arg(short, long, value_name = "OUTPUT")]
    output: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let mut compiler = Compiler::new(args.input.clone())?;
    let _llvm_module = compiler.compile()?;

    Ok(())
}
