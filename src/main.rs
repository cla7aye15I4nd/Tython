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

    let mut compiler = Compiler::new(args.input.clone(), args.output)?;

    let executable_path = compiler.compile()?;
    let output = std::process::Command::new(&executable_path).output()?;

    std::io::Write::write_all(&mut std::io::stdout(), &output.stdout)?;
    std::io::Write::write_all(&mut std::io::stderr(), &output.stderr)?;

    std::process::exit(output.status.code().unwrap_or(1));
}
