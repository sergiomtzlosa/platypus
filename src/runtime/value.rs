use std::fmt;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
    Array(Vec<Value>),
    Function {
        params: Vec<String>,
        body: Vec<crate::parser::ast::Stmt>,
        closure: HashMap<String, Value>,
    },
    Lambda {
        params: Vec<String>,
        body: Box<crate::parser::ast::Expr>,
        closure: HashMap<String, Value>,
    },
    NativeFunction {
        name: String,
        arity: usize,
    },
    Class {
        name: String,
        parent: Option<Box<Value>>,
        methods: HashMap<String, (Vec<String>, Vec<crate::parser::ast::Stmt>)>, // method_name -> (params, body)
        properties: HashMap<String, Value>, // default properties
    },
    Object {
        class_name: String,
        properties: HashMap<String, Value>,
    },
    Null,
}

impl Value {
    pub fn type_name(&self) -> &str {
        match self {
            Value::Number(_) => "Number",
            Value::String(_) => "String",
            Value::Boolean(_) => "Boolean",
            Value::Array(_) => "Array",
            Value::Function { .. } => "Function",
            Value::Lambda { .. } => "Function",
            Value::NativeFunction { .. } => "Function",
            Value::Class { .. } => "Class",
            Value::Object { class_name: _, .. } => "Object",
            Value::Null => "Null",
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Null => false,
            Value::Boolean(b) => *b,
            Value::Number(n) => *n != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Array(arr) => !arr.is_empty(),
            _ => true,
        }
    }

    pub fn to_number(&self) -> Result<f64, String> {
        match self {
            Value::Number(n) => Ok(*n),
            Value::String(s) => s.parse::<f64>().map_err(|_| format!("Cannot convert '{}' to number", s)),
            Value::Boolean(b) => Ok(if *b { 1.0 } else { 0.0 }),
            _ => Err(format!("Cannot convert {} to number", self.type_name())),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Number(n) => {
                if n.fract() == 0.0 {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{}", n)
                }
            }
            Value::String(s) => write!(f, "{}", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Array(arr) => {
                write!(f, "[")?;
                for (i, v) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
            Value::Function { params, .. } => write!(f, "<function({})>", params.len()),
            Value::Lambda { params, .. } => write!(f, "<lambda({})>", params.len()),
            Value::NativeFunction { name, arity } => write!(f, "<native function {}({})>", name, arity),
            Value::Class { name, .. } => write!(f, "<class {}>", name),
            Value::Object { class_name, .. } => write!(f, "<{} object>", class_name),
            Value::Null => write!(f, "null"),
        }
    }
}
