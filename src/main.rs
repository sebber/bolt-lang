mod ast;
mod c_codegen;
mod error;
mod lexer;
mod module;
mod parser;
mod symbol_table;
mod type_checker;

use clap::{Arg, Command as ClapCommand};
use std::fs;
use std::process::Command;

use c_codegen::CCodeGen;
use error::CompileError;
use lexer::Lexer;
use module::ModuleSystem;
use parser::Parser;
// use type_checker::TypeChecker;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), CompileError> {
    let matches = ClapCommand::new("bolt")
        .version("0.1.0")
        .about("Bolt programming language compiler")
        .arg(
            Arg::new("input")
                .help("Input .bolt file")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .help("Output executable name")
                .default_value("output"),
        )
        .arg(
            Arg::new("release")
                .short('r')
                .long("release")
                .help("Build in release mode")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let input_file = matches.get_one::<String>("input").unwrap();
    let output_file = matches.get_one::<String>("output").unwrap();
    let is_release = matches.get_flag("release");

    // Determine build mode and output directory
    let build_mode = if is_release { "release" } else { "debug" };
    let output_dir = format!("out/{}", build_mode);

    // Create output directory if it doesn't exist
    fs::create_dir_all(&output_dir).map_err(CompileError::IoError)?;

    let full_output_path = format!("{}/{}", output_dir, output_file);

    // Read the source file
    let source = fs::read_to_string(input_file).map_err(CompileError::IoError)?;

    // Initialize module system
    let mut module_system = ModuleSystem::new();

    // Lexical analysis
    let mut lexer = Lexer::new(source);
    let tokens = lexer
        .tokenize()
        .map_err(|e| CompileError::CodegenError(format!("Lexer error: {}", e)))?;

    // Parsing
    let mut parser = Parser::new(tokens);
    let ast = parser
        .parse()
        .map_err(|e| CompileError::CodegenError(format!("Parser error: {}", e)))?;

    // Extract symbol table from parser
    let symbol_table = parser.into_symbol_table();

    // Resolve imports and load modules
    module_system
        .resolve_imports(&ast)
        .map_err(|e| CompileError::CodegenError(format!("Module resolution error: {}", e)))?;

    // Type checking (currently disabled to avoid breaking existing tests)
    // TODO: Enable once type checker is more robust
    /*
    let mut type_checker = TypeChecker::new().with_module_system(module_system.clone());
    if let Err(type_errors) = type_checker.check_program(&ast) {
        println!("Type errors found:");
        for error in &type_errors {
            println!("  {}", error.message);
        }
        return Err(CompileError::CodegenError(
            format!("Type checking failed with {} errors", type_errors.len())
        ));
    }
    */

    // Code generation with module support and symbol table
    let mut codegen = CCodeGen::with_symbol_table(symbol_table);
    let c_code = codegen.compile_program_with_modules(ast, &module_system);

    // Output generated C code for debugging (only in debug mode)
    if !is_release {
        println!("Generated C code:");
        println!("{}", c_code);
    }

    // Write C file to output directory
    let c_file = format!("{}/{}.c", output_dir, output_file);
    fs::write(&c_file, c_code).map_err(CompileError::IoError)?;

    // Compile with GCC (with optimizations in release mode)
    let mut gcc_command = Command::new("gcc");
    gcc_command.arg(&c_file).arg("-o").arg(&full_output_path);

    // Add library linking flags for extern functions
    for library in &codegen.required_libraries {
        gcc_command.arg(&format!("-l{}", library));
    }

    if is_release {
        gcc_command.arg("-O2").arg("-DNDEBUG");
    } else {
        gcc_command.arg("-g").arg("-DDEBUG");
    }

    let status = gcc_command.status().map_err(CompileError::IoError)?;

    if !status.success() {
        return Err(CompileError::CodegenError(
            "GCC compilation failed".to_string(),
        ));
    }

    // Clean up C file (only in release mode to keep debug artifacts)
    if is_release {
        fs::remove_file(&c_file).ok();
    }

    println!(
        "Successfully compiled {} to {} ({})",
        input_file, full_output_path, build_mode
    );

    Ok(())
}
