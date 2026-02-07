mod lexer;
mod parser;
mod runtime;

use std::env;
use std::fs;
use std::io::{self, Write};
use std::process;

use lexer::Lexer;
use parser::Parser;
use runtime::Interpreter;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    match args[1].as_str() {
        "run" => {
            if args.len() < 3 {
                eprintln!("Error: No input file provided");
                print_usage();
                process::exit(1);
            }
            run_file(&args[2]);
        }
        "repl" => {
            run_repl();
        }
        "--help" | "-h" => {
            print_usage();
        }
        "--version" | "-v" => {
            println!("Platypus v0.1.0");
        }
        _ => {
            eprintln!("Error: Unknown command '{}'", args[1]);
            print_usage();
            process::exit(1);
        }
    }
}

fn print_usage() {
    println!("Platypus Programming Language v0.1.0");
    println!();
    println!("USAGE:");
    println!("    platypus <COMMAND> [OPTIONS]");
    println!();
    println!("COMMANDS:");
    println!("    run <file>     Compile and execute a Platypus source file");
    println!("    repl           Start an interactive REPL");
    println!("    --help, -h     Print this help message");
    println!("    --version, -v  Print version information");
    println!();
    println!("EXAMPLES:");
    println!("    platypus run hello.plat");
    println!("    platypus repl");
}

fn run_file(filename: &str) {
    let source = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading file '{}': {}", filename, err);
            process::exit(1);
        }
    };

    if let Err(err) = execute_source(&source) {
        eprintln!("Error: {}", err);
        process::exit(1);
    }
}

fn run_repl() {
    println!("Platypus REPL v0.1.0");
    println!("Type 'exit' or press Ctrl+D to quit");
    println!();

    let mut interpreter = Interpreter::new();
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!(">> ");
        stdout.flush().unwrap();

        let mut input = String::new();
        match stdin.read_line(&mut input) {
            Ok(0) => break, // EOF
            Ok(_) => {
                let input = input.trim();
                if input == "exit" {
                    break;
                }
                if input.is_empty() {
                    continue;
                }

                // Try to parse and execute
                match execute_repl_line(&mut interpreter, input) {
                    Ok(Some(value)) => {
                        // Only print if it's not null
                        if !matches!(value, runtime::value::Value::Null) {
                            println!("{}", value);
                        }
                    }
                    Ok(None) => {}
                    Err(err) => eprintln!("Error: {}", err),
                }
            }
            Err(err) => {
                eprintln!("Error reading input: {}", err);
                break;
            }
        }
    }

    println!("Goodbye!");
}

fn execute_source(source: &str) -> Result<(), String> {
    // Lexing
    let mut lexer = Lexer::new(source.to_string());
    let tokens = lexer.tokenize()?;

    // Parsing
    let mut parser = Parser::new(tokens);
    let program = parser.parse()?;

    // Execution
    let mut interpreter = Interpreter::new();
    interpreter.execute(&program)?;

    Ok(())
}

fn execute_repl_line(interpreter: &mut Interpreter, source: &str) -> Result<Option<runtime::value::Value>, String> {
    // Lexing
    let mut lexer = Lexer::new(source.to_string());
    let tokens = lexer.tokenize()?;

    // Parsing
    let mut parser = Parser::new(tokens);
    let program = parser.parse()?;

    // For REPL, if there's a single expression statement, return its value
    if program.statements.len() == 1 {
        if let parser::ast::Stmt::Expr(expr) = &program.statements[0] {
            return Ok(Some(interpreter.evaluate_expr(expr)?));
        }
    }

    // Otherwise execute normally
    interpreter.execute(&program)?;
    Ok(None)
}
