use camino::Utf8PathBuf;
use clap::{arg, Arg, Command};
use compiler::Compiler;
use triton_vm::prelude::*;

mod asm_parser;
mod compiler;
mod log;
mod parser;
mod vm;

fn cli() -> Command {
    Command::new("acc")
        .about("ashlang compiler")
        .subcommand_required(false)
        .arg_required_else_help(true)
        .arg(arg!(<ENTRY_FN> "The entrypoint function name"))
        .arg(
            Arg::new("include")
                .short('i')
                .long("include")
                .required(false)
                .help("specify a path to be recursively included")
                .action(clap::ArgAction::Append),
        )
        .arg(
            Arg::new("print_asm")
                .short('v')
                .long("asm")
                .required(false)
                .num_args(0)
                .help("print the compiled asm before proving"),
        )
        .arg(
            Arg::new("public_inputs")
                .short('p')
                .long("public")
                .required(false)
                .help("public inputs to the program"),
        )
        .arg(
            Arg::new("secret_inputs")
                .short('s')
                .long("secret")
                .required(false)
                .help("secret inputs to the program"),
        )
}

fn parse_inputs(inputs: Option<&String>) -> Vec<BFieldElement> {
    if let Some(i) = inputs {
        i.split(',')
            .filter(|v| !v.is_empty())
            .map(|v| v.parse().unwrap())
            .collect()
    } else {
        vec![]
    }
}

fn main() {
    let matches = cli().get_matches();
    let entry_fn = matches
        .get_one::<String>("ENTRY_FN")
        .expect("Failed to get ENTRY_FN");
    let include_paths = matches
        .get_many::<String>("include")
        .unwrap_or_default()
        .map(|v| v.as_str())
        .collect::<Vec<_>>();
    let public_inputs = matches.get_one::<String>("public_inputs");
    let secret_inputs = matches.get_one::<String>("secret_inputs");
    let mut compiler = Compiler::new();
    for p in include_paths {
        if p.is_empty() {
            continue;
        }
        compiler.include(Utf8PathBuf::from(p));
    }

    compiler.print_asm = *matches.get_one::<bool>("print_asm").unwrap_or(&false);
    let asm = compiler.compile(entry_fn);

    let instructions = triton_vm::parser::parse(&asm).unwrap();
    let l_instructions = triton_vm::parser::to_labelled_instructions(instructions.as_slice());
    let program = triton_vm::program::Program::new(l_instructions.as_slice());

    let public_inputs = PublicInput::from(parse_inputs(public_inputs));
    let secret_inputs = NonDeterminism::from(parse_inputs(secret_inputs));
    match triton_vm::prove_program(&program, public_inputs, secret_inputs) {
        Ok((_stark, _claim, _proof)) => {
            println!("{:?}", _stark);
            println!("{:?}", _claim);
        }
        Err(e) => {
            println!("Triton VM errored");
            println!("{e}");
        }
    }
}
