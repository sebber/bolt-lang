pub mod expressions;
pub mod monomorphization;
pub mod statements;
pub mod types;

// Re-export commonly used types
// Note: These are available for when c_codegen.rs is refactored to use them
// pub use monomorphization::{MonomorphicType, Monomorphizer};
// pub use types::{type_to_c_string, type_to_simple_name, get_c_type_default};
// pub use expressions::ExpressionCompiler;
// pub use statements::StatementCompiler;
