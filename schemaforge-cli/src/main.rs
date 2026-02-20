use clap::{Parser, Subcommand};
use schemaforge::registry;
use schemaforge::Error;
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "schemaforge")]
#[command(about = "Schemaforge CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    ListPasses,
    RunPass {
        pass: String,
        #[arg(long = "in")]
        input: PathBuf,
        #[arg(long = "out")]
        output: PathBuf,
    },
}

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {}", err);
        std::process::exit(1);
    }
}

fn run() -> Result<(), Error> {
    let cli = Cli::parse();

    match cli.command {
        Commands::ListPasses => {
            for pass in registry::all_passes() {
                println!("{}\t{}", pass.name, pass.help);
            }
        }
        Commands::RunPass {
            pass,
            input,
            output,
        } => {
            let spec = registry::find_pass(&pass).ok_or_else(|| {
                Error::Pass(format!("unknown pass '{}'", pass))
            })?;
            let input_text = read_input(&input)?;
            let result = (spec.run)(&input_text)?;
            write_output(&output, &result)?;
        }
    }

    Ok(())
}

fn read_input(path: &PathBuf) -> Result<String, Error> {
    if path.as_os_str() == "-" {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        return Ok(buf);
    }

    Ok(fs::read_to_string(path)?)
}

fn write_output(path: &PathBuf, contents: &str) -> Result<(), Error> {
    if path.as_os_str() == "-" {
        let mut stdout = io::stdout();
        stdout.write_all(contents.as_bytes())?;
        return Ok(());
    }

    fs::write(path, contents)?;
    Ok(())
}
