use clap::Parser;
use std::path::PathBuf;
use tython::compiler::Compiler;
use tython::errors::print_error;

#[derive(Parser, Debug)]
#[command(name = "tython")]
#[command(about = "A Python to native code compiler", long_about = None)]
struct Args {
    #[arg(value_name = "FILE")]
    input: PathBuf,
}

fn main() {
    let args = Args::parse();

    let mut compiler = match Compiler::new(args.input.clone()) {
        Ok(c) => c,
        Err(e) => {
            print_error(&args.input, &e);
            std::process::exit(1);
        }
    };

    let exe_path = args
        .input
        .canonicalize()
        .unwrap()
        .with_extension(std::env::consts::EXE_EXTENSION);

    if let Err(e) = compiler.compile(exe_path.clone()) {
        print_error(&args.input, &e);
        std::process::exit(1);
    }

    let output = std::process::Command::new(&exe_path)
        .output()
        .expect("failed to execute compiled binary");

    std::io::Write::write_all(&mut std::io::stdout(), &output.stdout).ok();
    std::io::Write::write_all(&mut std::io::stderr(), &output.stderr).ok();

    std::process::exit(output.status.code().unwrap_or(1));
}
