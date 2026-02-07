use crate::runtime::value::Value;

pub fn register_builtins() -> std::collections::HashMap<String, Value> {
    let mut builtins = std::collections::HashMap::new();

    builtins.insert(
        "typeof".to_string(),
        Value::NativeFunction {
            name: "typeof".to_string(),
            arity: 1,
        },
    );

    builtins.insert(
        "print".to_string(),
        Value::NativeFunction {
            name: "print".to_string(),
            arity: 1,
        },
    );

    builtins.insert(
        "map".to_string(),
        Value::NativeFunction {
            name: "map".to_string(),
            arity: 2,
        },
    );

    builtins.insert(
        "filter".to_string(),
        Value::NativeFunction {
            name: "filter".to_string(),
            arity: 2,
        },
    );

    builtins.insert(
        "len".to_string(),
        Value::NativeFunction {
            name: "len".to_string(),
            arity: 1,
        },
    );

    builtins
}

pub fn call_builtin(name: &str, args: Vec<Value>) -> Result<Value, String> {
    match name {
        "typeof" => {
            if args.len() != 1 {
                return Err(format!("typeof expects 1 argument, got {}", args.len()));
            }
            Ok(Value::String(args[0].type_name().to_string()))
        }
        "print" => {
            if args.len() != 1 {
                return Err(format!("print expects 1 argument, got {}", args.len()));
            }
            println!("{}", args[0]);
            Ok(Value::Null)
        }
        "len" => {
            if args.len() != 1 {
                return Err(format!("len expects 1 argument, got {}", args.len()));
            }
            match &args[0] {
                Value::Array(arr) => Ok(Value::Number(arr.len() as f64)),
                Value::String(s) => Ok(Value::Number(s.len() as f64)),
                _ => Err(format!("len expects Array or String, got {}", args[0].type_name())),
            }
        }
        _ => Err(format!("Unknown builtin function: {}", name)),
    }
}
