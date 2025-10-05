//! Compilation pipeline stages.

use anyhow::{Context, Result};
use miette::NamedSource;
use rive_codegen::CodeGenerator;
use rive_core::{Span, type_system::TypeRegistry};
use rive_ir::{AstLowering, RirModule};
use rive_lexer::{Token, tokenize};
use rive_parser::{Program, parse};

/// Runs lexical analysis on source code.
///
/// # Errors
/// Returns an error if lexical analysis fails.
pub fn lex(source: &str) -> Result<Vec<(Token, Span)>> {
    tokenize(source).map_err(|e| {
        let report = miette::Report::new(e)
            .with_source_code(NamedSource::new("main.rive", source.to_string()));
        eprintln!("{report:?}");
        anyhow::anyhow!("Lexical analysis failed")
    })
}

/// Runs parsing on tokens to produce an AST and type registry.
///
/// # Errors
/// Returns an error if parsing fails.
pub fn parse_tokens(tokens: &[(Token, Span)], source: &str) -> Result<(Program, TypeRegistry)> {
    parse(tokens).map_err(|e| {
        let report = miette::Report::new(e)
            .with_source_code(NamedSource::new("main.rive", source.to_string()));
        eprintln!("{report:?}");
        anyhow::anyhow!("Parsing failed")
    })
}

/// Runs semantic analysis on the AST.
///
/// # Errors
/// Returns an error if semantic analysis fails.
pub fn analyze(program: &Program, type_registry: TypeRegistry, source: &str) -> Result<()> {
    rive_semantic::analyze_with_registry(program, type_registry).map_err(|e| {
        let report = miette::Report::new(e)
            .with_source_code(NamedSource::new("main.rive", source.to_string()));
        eprintln!("{report:?}");
        anyhow::anyhow!("Semantic analysis failed")
    })
}

/// Lowers AST to RIR (Rive Intermediate Representation).
///
/// # Errors
/// Returns an error if lowering fails.
pub fn lower(program: &Program, type_registry: TypeRegistry, source: &str) -> Result<RirModule> {
    let mut lowering = AstLowering::new(type_registry);
    lowering.lower_program(program).map_err(|e| {
        let report = miette::Report::new(e)
            .with_source_code(NamedSource::new("main.rive", source.to_string()));
        eprintln!("{report:?}");
        anyhow::anyhow!("RIR lowering failed")
    })
}

/// Generates Rust code from RIR.
///
/// # Errors
/// Returns an error if code generation fails.
pub fn generate(rir_module: &RirModule) -> Result<String> {
    let mut codegen = CodeGenerator::new();
    codegen
        .generate(rir_module)
        .with_context(|| "Code generation failed")
}

/// Runs the complete compilation pipeline for checking (no code generation).
///
/// # Errors
/// Returns an error if any stage fails.
pub fn check_pipeline(source: &str) -> Result<()> {
    let tokens = lex(source)?;
    let (ast, type_registry) = parse_tokens(&tokens, source)?;
    analyze(&ast, type_registry.clone(), source)?;
    let rir_module = lower(&ast, type_registry, source)?;
    let _rust_code = generate(&rir_module)?;
    Ok(())
}

/// Runs the complete compilation pipeline and returns generated code.
///
/// # Errors
/// Returns an error if any stage fails.
pub fn build_pipeline(source: &str) -> Result<String> {
    let tokens = lex(source)?;
    let (ast, type_registry) = parse_tokens(&tokens, source)?;
    analyze(&ast, type_registry.clone(), source)?;
    let rir_module = lower(&ast, type_registry, source)?;
    generate(&rir_module)
}
