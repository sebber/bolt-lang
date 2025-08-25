use crate::ast::{Field, Type};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MonomorphicType {
    pub base_name: String,      // e.g., "Array", "Map"
    pub type_args: Vec<String>, // e.g., ["Integer"], ["String", "Integer"]
}

impl MonomorphicType {
    pub fn new(base_name: String, type_args: Vec<String>) -> Self {
        Self {
            base_name,
            type_args,
        }
    }

    pub fn mangled_name(&self) -> String {
        if self.type_args.is_empty() {
            self.base_name.clone()
        } else {
            format!("{}_{}", self.base_name, self.type_args.join("_"))
        }
    }
}

pub struct Monomorphizer {
    generic_types: HashMap<String, (Vec<String>, Vec<Field>)>, // base_name -> (type_params, fields)
    required_monomorphs: HashSet<MonomorphicType>, // Track which concrete types are needed
    generated_monomorphs: HashMap<MonomorphicType, String>, // Cache generated C code
}

impl Default for Monomorphizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Monomorphizer {
    pub fn new() -> Self {
        Self {
            generic_types: HashMap::new(),
            required_monomorphs: HashSet::new(),
            generated_monomorphs: HashMap::new(),
        }
    }

    // Add a generic type definition to the registry
    pub fn register_generic_type(
        &mut self,
        name: String,
        type_params: Vec<String>,
        fields: Vec<Field>,
    ) {
        self.generic_types.insert(name, (type_params, fields));
    }

    // Mark a concrete generic type as needed for generation
    pub fn require_monomorph(&mut self, base_name: String, type_args: Vec<String>) {
        let monomorph = MonomorphicType::new(base_name, type_args);
        self.required_monomorphs.insert(monomorph);
    }

    // Generate C struct for a specific monomorphic type
    pub fn generate_monomorphic_struct(
        &mut self,
        monomorph: &MonomorphicType,
        type_to_c: impl Fn(&Type) -> String,
    ) -> String {
        if let Some(cached) = self.generated_monomorphs.get(monomorph) {
            return cached.clone();
        }

        if let Some((type_params, fields)) = self.generic_types.get(&monomorph.base_name) {
            let mut result = String::new();
            let struct_name = monomorph.mangled_name();

            result.push_str("typedef struct {\n");

            for field in fields {
                let concrete_type =
                    substitute_type_params(&field.field_type, type_params, &monomorph.type_args);
                let field_type_str = type_to_c(&concrete_type);
                result.push_str(&format!("    {} {};\n", field_type_str, field.name));
            }

            result.push_str(&format!("}} {};\n\n", struct_name));

            self.generated_monomorphs
                .insert(monomorph.clone(), result.clone());
            result
        } else {
            panic!("Unknown generic type: {}", monomorph.base_name);
        }
    }

    pub fn get_required_monomorphs(&self) -> &HashSet<MonomorphicType> {
        &self.required_monomorphs
    }

    pub fn has_generic_type(&self, name: &str) -> bool {
        self.generic_types.contains_key(name)
    }
}

// Substitute type parameters with concrete types
pub fn substitute_type_params(
    generic_type: &Type,
    type_params: &[String],
    concrete_types: &[String],
) -> Type {
    match generic_type {
        Type::Custom(name) => {
            // Check if this is a type parameter
            if let Some(index) = type_params.iter().position(|p| p == name) {
                if let Some(concrete) = concrete_types.get(index) {
                    Type::Custom(concrete.clone())
                } else {
                    Type::Custom(name.clone())
                }
            } else {
                Type::Custom(name.clone())
            }
        }
        Type::Pointer(inner) => Type::Pointer(Box::new(substitute_type_params(
            inner,
            type_params,
            concrete_types,
        ))),
        Type::Generic {
            name,
            type_params: generic_params,
        } => {
            let substituted_args: Vec<Type> = generic_params
                .iter()
                .map(|arg| substitute_type_params(arg, type_params, concrete_types))
                .collect();
            Type::Generic {
                name: name.clone(),
                type_params: substituted_args,
            }
        }
        Type::TypeParameter(param) => {
            if let Some(index) = type_params.iter().position(|p| p == param) {
                if let Some(concrete) = concrete_types.get(index) {
                    Type::Custom(concrete.clone())
                } else {
                    Type::TypeParameter(param.clone())
                }
            } else {
                Type::TypeParameter(param.clone())
            }
        }
        _ => generic_type.clone(),
    }
}
