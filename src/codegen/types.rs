use crate::ast::Type;
use super::monomorphization::MonomorphicType;

/// Convert an AST Type to its C equivalent string representation
pub fn type_to_c_string(t: &Type) -> String {
    match t {
        Type::Integer => "int".to_string(),
        Type::String => "char*".to_string(),
        Type::Bool => "int".to_string(),
        Type::Pointer(inner) => format!("{}*", type_to_c_string(inner.as_ref())),
        Type::Custom(name) => name.clone(),
        Type::Generic { name, type_params } => {
            // Generate monomorphic type name
            let type_arg_names: Vec<String> = type_params.iter()
                .map(|t| type_to_simple_name(t))
                .collect();
            MonomorphicType::new(name.clone(), type_arg_names).mangled_name()
        }
        Type::TypeParameter(param) => param.clone(),
        Type::Array(inner_type) => format!("{}*", type_to_c_string(inner_type)),
    }
}

/// Convert a Type to a simple name string (for use in monomorphic type names)
pub fn type_to_simple_name(t: &Type) -> String {
    match t {
        Type::Integer => "Integer".to_string(),
        Type::String => "String".to_string(),
        Type::Bool => "Bool".to_string(),
        Type::Custom(n) => n.clone(),
        Type::Generic { name, type_params } => {
            if type_params.is_empty() {
                name.clone()
            } else {
                let args = type_params.iter()
                    .map(|t| type_to_simple_name(t))
                    .collect::<Vec<_>>()
                    .join("_");
                format!("{}_{}", name, args)
            }
        }
        Type::Pointer(inner) => format!("Ptr_{}", type_to_simple_name(inner)),
        Type::TypeParameter(param) => param.clone(),
        Type::Array(inner_type) => format!("Array_{}", type_to_simple_name(inner_type)),
    }
}

/// Get the default value for a C type
pub fn get_c_type_default(t: &Type) -> String {
    match t {
        Type::Integer | Type::Bool => "0".to_string(),
        Type::String => "\"\"".to_string(),
        Type::Pointer(_) => "NULL".to_string(),
        Type::Custom(_) | Type::Generic { .. } => "{}".to_string(),
        Type::TypeParameter(_) => "0".to_string(),
        Type::Array(_) => "NULL".to_string(),
    }
}