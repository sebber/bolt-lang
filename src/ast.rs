#[derive(Debug, Clone)]
pub enum Type {
    String,
    Integer,
    Bool,
    #[allow(dead_code)] // For future typed arrays
    Array(Box<Type>), // Array of some type
    Pointer(Box<Type>), // Pointer to some type (^Type)
    #[allow(dead_code)] // For future custom types
    Custom(String),
    // Generic types like Array[T], Map[K, V]
    Generic {
        name: String,           // e.g., "Array", "Map"
        type_params: Vec<Type>, // e.g., [Integer], [String, Integer]
    },
    // Type parameters like T, K, V
    TypeParameter(String),
}

#[derive(Debug, Clone)]
pub struct Field {
    #[allow(dead_code)] // For future struct definitions
    pub name: String,
    #[allow(dead_code)] // For future struct definitions
    pub field_type: Type,
}

#[derive(Debug, Clone)]
pub struct StructField {
    pub name: String,
    pub value: Expression,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub param_type: Type,
}

#[derive(Debug, Clone)]
pub enum Statement {
    VarDecl {
        name: String,
        #[allow(dead_code)] // For future type checking
        type_annotation: Option<Type>,
        value: Expression,
    },
    ValDecl {
        name: String,
        #[allow(dead_code)] // For future type checking
        type_annotation: Option<Type>,
        value: Expression,
    },
    #[allow(dead_code)] // For future struct definitions
    TypeDef {
        name: String,
        type_params: Vec<String>, // Generic parameters like ["T", "K", "V"]
        fields: Vec<Field>,
    },
    If {
        condition: Expression,
        then_body: Vec<Statement>,
        else_body: Option<Vec<Statement>>,
    },
    ForIn {
        variable: String,
        iterable: Expression,
        body: Vec<Statement>,
    },
    ForCondition {
        condition: Expression,
        body: Vec<Statement>,
    },
    ForLoop {
        init: Option<Box<Statement>>,
        condition: Option<Expression>,
        update: Option<Expression>,
        body: Vec<Statement>,
    },
    Import {
        module_name: Option<String>, // Some("module") for namespace import, None for selective
        module_path: String,
        items: Option<Vec<String>>, // None means import all as namespace, Some([items]) means selective
    },
    Export {
        item: String, // Single item export
    },
    Function {
        name: String,
        params: Vec<Parameter>,
        return_type: Option<Type>,
        body: Vec<Statement>,
        exported: bool,
    },
    Return(Option<Expression>),
    Expression(Expression),
    Assignment {
        variable: String,
        value: Expression,
    },
}

#[derive(Debug, Clone)]
pub enum Expression {
    StringLiteral(String),
    IntegerLiteral(i64),
    BoolLiteral(bool),
    ArrayLiteral(Vec<Expression>),
    Identifier(String),
    FunctionCall {
        name: String,
        args: Vec<Expression>,
    },
    NamespacedFunctionCall {
        namespace: String,
        function: String,
        args: Vec<Expression>,
    },
    BinaryOp {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
    },
    UnaryOp {
        operator: UnaryOperator,
        operand: Box<Expression>,
    },
    StructLiteral {
        type_name: String,
        type_args: Option<Vec<Type>>, // For generic constructors like Array[Integer]
        fields: Vec<StructField>,
    },
    FieldAccess {
        object: Box<Expression>,
        field: String,
    },
    ArrayAccess {
        array: Box<Expression>,
        index: Box<Expression>,
    },
    AddressOf {
        operand: Box<Expression>,
    },
    Dereference {
        operand: Box<Expression>,
    },
}

#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Not,
}

#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Or,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Statement>,
}