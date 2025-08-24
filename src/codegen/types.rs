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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_type_to_c_string() {
        assert_eq!(type_to_c_string(&Type::Integer), "int");
        assert_eq!(type_to_c_string(&Type::String), "char*");
        assert_eq!(type_to_c_string(&Type::Bool), "int");
    }

    #[test]
    fn test_custom_type_to_c_string() {
        assert_eq!(type_to_c_string(&Type::Custom("MyType".to_string())), "MyType");
    }

    #[test]
    fn test_pointer_type_to_c_string() {
        let inner = Type::Integer;
        let pointer = Type::Pointer(Box::new(inner));
        assert_eq!(type_to_c_string(&pointer), "int*");
        
        // Nested pointer
        let double_pointer = Type::Pointer(Box::new(pointer));
        assert_eq!(type_to_c_string(&double_pointer), "int**");
    }

    #[test]
    fn test_array_type_to_c_string() {
        let array = Type::Array(Box::new(Type::Integer));
        assert_eq!(type_to_c_string(&array), "int*");
    }

    #[test]
    fn test_generic_type_to_c_string() {
        let generic = Type::Generic {
            name: "Array".to_string(),
            type_params: vec![Type::Integer],
        };
        assert_eq!(type_to_c_string(&generic), "Array_Integer");
    }

    #[test]
    fn test_type_parameter_to_c_string() {
        let param = Type::TypeParameter("T".to_string());
        assert_eq!(type_to_c_string(&param), "T");
    }

    #[test]
    fn test_type_to_simple_name() {
        assert_eq!(type_to_simple_name(&Type::Integer), "Integer");
        assert_eq!(type_to_simple_name(&Type::String), "String");
        assert_eq!(type_to_simple_name(&Type::Bool), "Bool");
        assert_eq!(type_to_simple_name(&Type::Custom("Foo".to_string())), "Foo");
    }

    #[test]
    fn test_generic_type_simple_name() {
        let generic = Type::Generic {
            name: "Result".to_string(),
            type_params: vec![Type::String, Type::Integer],
        };
        assert_eq!(type_to_simple_name(&generic), "Result_String_Integer");
    }

    #[test]
    fn test_pointer_simple_name() {
        let pointer = Type::Pointer(Box::new(Type::String));
        assert_eq!(type_to_simple_name(&pointer), "Ptr_String");
    }

    #[test]
    fn test_array_simple_name() {
        let array = Type::Array(Box::new(Type::Bool));
        assert_eq!(type_to_simple_name(&array), "Array_Bool");
    }

    #[test]
    fn test_c_type_defaults() {
        assert_eq!(get_c_type_default(&Type::Integer), "0");
        assert_eq!(get_c_type_default(&Type::Bool), "0");
        assert_eq!(get_c_type_default(&Type::String), r#""""#);
        assert_eq!(get_c_type_default(&Type::Pointer(Box::new(Type::Integer))), "NULL");
        assert_eq!(get_c_type_default(&Type::Array(Box::new(Type::Integer))), "NULL");
        assert_eq!(get_c_type_default(&Type::Custom("Foo".to_string())), "{}");
    }
}