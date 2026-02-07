pub mod value;
pub mod builtins;

use crate::parser::ast::*;
use value::Value;
use std::collections::HashMap;

pub struct Interpreter {
    globals: HashMap<String, Value>,
    scopes: Vec<HashMap<String, Value>>,
    in_context: bool, // Track if we're executing within a function or method
}

impl Interpreter {
    pub fn new() -> Self {
        let globals = builtins::register_builtins();
        Interpreter {
            globals,
            scopes: Vec::new(),
            in_context: false,
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn get_variable(&self, name: &str) -> Result<Value, String> {
        // Search in scopes from innermost to outermost
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Ok(value.clone());
            }
        }

        // Search in globals
        if let Some(value) = self.globals.get(name) {
            return Ok(value.clone());
        }

        Err(format!("Undefined variable: {}", name))
    }

    fn set_variable(&mut self, name: String, value: Value) {
        // Try to update in scopes first
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(&name) {
                scope.insert(name, value);
                return;
            }
        }

        // Set in global scope
        self.globals.insert(name, value);
    }

    fn define_variable(&mut self, name: String, value: Value) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, value);
        } else {
            self.globals.insert(name, value);
        }
    }

    pub fn execute(&mut self, program: &Program) -> Result<(), String> {
        for stmt in &program.statements {
            self.execute_stmt(stmt)?;
        }
        Ok(())
    }

    fn execute_stmt(&mut self, stmt: &Stmt) -> Result<Option<Value>, String> {
        match stmt {
            Stmt::VarDecl { name, value } => {
                let val = self.evaluate_expr(value)?;
                // Check if variable already exists; if so, update it; otherwise, create new one
                if self.get_variable(name).is_ok() {
                    self.set_variable(name.clone(), val);
                } else {
                    self.define_variable(name.clone(), val);
                }
                Ok(None)
            }
            Stmt::FuncDecl { name, params, body, .. } => {
                let closure = self.capture_closure();
                let func = Value::Function {
                    params: params.clone(),
                    body: body.clone(),
                    closure,
                };
                self.define_variable(name.clone(), func);
                Ok(None)
            }
            Stmt::Return(expr) => {
                let val = if let Some(e) = expr {
                    self.evaluate_expr(e)?
                } else {
                    Value::Null
                };
                Ok(Some(val))
            }
            Stmt::Expr(expr) => {
                self.evaluate_expr(expr)?;
                Ok(None)
            }
            Stmt::If { condition, then_branch, else_branch } => {
                let cond_val = self.evaluate_expr(condition)?;
                if cond_val.is_truthy() {
                    self.execute_stmt(then_branch)
                } else if let Some(else_stmt) = else_branch {
                    self.execute_stmt(else_stmt)
                } else {
                    Ok(None)
                }
            }
            Stmt::While { condition, body } => {
                while self.evaluate_expr(condition)?.is_truthy() {
                    if let Some(val) = self.execute_stmt(body)? {
                        return Ok(Some(val));
                    }
                }
                Ok(None)
            }
            Stmt::For { init, condition, increment, body } => {
                // Execute initializer
                if let Some(init_stmt) = init {
                    self.execute_stmt(init_stmt)?;
                }

                // Loop while condition is true
                loop {
                    // Check condition
                    if let Some(cond) = condition {
                        if !self.evaluate_expr(cond)?.is_truthy() {
                            break;
                        }
                    }

                    // Execute body
                    if let Some(val) = self.execute_stmt(body)? {
                        return Ok(Some(val));
                    }

                    // Execute increment
                    if let Some(inc) = increment {
                        self.evaluate_expr(inc)?;
                    }
                }
                Ok(None)
            }
            Stmt::ForEach { variable, iterable, body } => {
                let iter_val = self.evaluate_expr(iterable)?;
                
                match iter_val {
                    Value::Array(items) => {
                        for item in items {
                            self.define_variable(variable.clone(), item);
                            if let Some(val) = self.execute_stmt(body)? {
                                return Ok(Some(val));
                            }
                        }
                        Ok(None)
                    }
                    _ => Err(format!("Cannot iterate over non-array value in foreach loop"))
                }
            }
            Stmt::Block(stmts) => {
                self.push_scope();
                let mut result = None;
                for stmt in stmts {
                    if let Some(val) = self.execute_stmt(stmt)? {
                        result = Some(val);
                        break;
                    }
                }
                self.pop_scope();
                Ok(result)
            }
            Stmt::ClassDecl { name, extends, methods, properties } => {
                // Build methods map
                let mut methods_map = HashMap::new();
                for (method_name, params, _return_type, body) in methods {
                    methods_map.insert(method_name.clone(), (params.clone(), body.clone()));
                }
                
                // Build properties map with defaults
                let mut properties_map = HashMap::new();
                for (prop_name, expr) in properties {
                    let val = self.evaluate_expr(expr)?;
                    properties_map.insert(prop_name.clone(), val);
                }
                
                // Get parent class if extending
                let parent_value = if let Some(parent_name) = extends {
                    match self.get_variable(parent_name) {
                        Ok(Value::Class { .. }) => Some(Box::new(self.get_variable(parent_name)?)),
                        _ => return Err(format!("Parent class '{}' not found", parent_name)),
                    }
                } else {
                    None
                };
                
                let class_value = Value::Class {
                    name: name.clone(),
                    parent: parent_value,
                    methods: methods_map,
                    properties: properties_map,
                };
                
                self.define_variable(name.clone(), class_value);
                Ok(None)
            }
        }
    }

    pub fn evaluate_expr(&mut self, expr: &Expr) -> Result<Value, String> {
        match expr {
            Expr::Literal(lit) => Ok(self.literal_to_value(lit)),
            Expr::Variable(name) => self.get_variable(name),
            Expr::Assign { name, value } => {
                let val = self.evaluate_expr(value)?;
                self.set_variable(name.clone(), val.clone());
                Ok(val)
            }
            Expr::PropertyAssign { object, property, value } => {
                let obj_val = self.evaluate_expr(object)?;
                let val = self.evaluate_expr(value)?;
                
                match obj_val {
                    Value::Object { class_name, mut properties } => {
                        // Check if property is private and we're not in a method
                        if property.starts_with("_") && !self.in_context {
                            return Err(format!("Cannot assign private property '{}' from outside class", property));
                        }
                        properties.insert(property.clone(), val.clone());
                        // Update the object in scope
                        if let Expr::Variable(var_name) = &**object {
                            self.set_variable(var_name.clone(), Value::Object { class_name, properties });
                        }
                        Ok(val)
                    }
                    _ => Err(format!("Cannot assign property to {}", obj_val.type_name())),
                }
            }
            Expr::BinaryOp { left, operator, right } => {
                let left_val = self.evaluate_expr(left)?;
                let right_val = self.evaluate_expr(right)?;
                self.apply_binary_op(&left_val, operator, &right_val)
            }
            Expr::UnaryOp { operator, right } => {
                let val = self.evaluate_expr(right)?;
                self.apply_unary_op(operator, &val)
            }
            Expr::FunctionCall { name, args } => {
                self.call_function(name, args)
            }
            Expr::Lambda { params, body } => {
                let closure = self.capture_closure();
                Ok(Value::Lambda {
                    params: params.clone(),
                    body: body.clone(),
                    closure,
                })
            }
            Expr::Match { expr, cases } => {
                let val = self.evaluate_expr(expr)?;
                self.match_value(&val, cases)
            }
            Expr::Array(elements) => {
                let mut arr = Vec::new();
                for elem in elements {
                    arr.push(self.evaluate_expr(elem)?);
                }
                Ok(Value::Array(arr))
            }
            Expr::New { class_name, args: _ } => {
                // Check if this is a private class and we're not in context
                if class_name.starts_with("_") && !self.in_context {
                    return Err(format!("Cannot instantiate private class '{}' from outside context", class_name));
                }

                match self.get_variable(class_name) {
                    Ok(Value::Class { properties, parent, .. }) => {
                        // Start with parent properties if extending
                        let mut obj_props = HashMap::new();
                        
                        if let Some(parent_class) = parent {
                            if let Value::Class { properties: parent_props, .. } = &*parent_class {
                                obj_props = parent_props.clone();
                            }
                        }
                        
                        // Override with own properties
                        for (name, val) in &properties {
                            obj_props.insert(name.clone(), val.clone());
                        }
                        
                        Ok(Value::Object {
                            class_name: class_name.clone(),
                            properties: obj_props,
                        })
                    }
                    _ => Err(format!("Class '{}' not found", class_name)),
                }
            }
            Expr::PropertyAccess { object, property } => {
                let obj_val = self.evaluate_expr(object)?;
                match obj_val {
                    Value::Object { properties, .. } => {
                        // Check if property is private and we're not in a method
                        if property.starts_with("_") && !self.in_context {
                            return Err(format!("Cannot access private property '{}' from outside class", property));
                        }
                        properties.get(property).cloned()
                            .ok_or_else(|| format!("Property '{}' not found on object", property))
                    }
                    _ => Err(format!("Cannot access property '{}' on {}", property, obj_val.type_name())),
                }
            }
            Expr::MethodCall { object, method, args } => {
                let obj_val = self.evaluate_expr(object)?;
                match &obj_val {
                    Value::Object { class_name, properties } => {
                        // Look up method in class
                        if let Ok(Value::Class { methods, .. }) = self.get_variable(class_name) {
                            if let Some((params, body)) = methods.get(method) {
                                // Call method with object as context
                                let mut method_scope = HashMap::new();
                                method_scope.insert("this".to_string(), obj_val.clone());
                                
                                // Add all properties from the object to the scope
                                for (prop_name, prop_val) in properties {
                                    method_scope.insert(prop_name.clone(), prop_val.clone());
                                }
                                
                                for (i, param) in params.iter().enumerate() {
                                    let arg_val = if i < args.len() {
                                        self.evaluate_expr(&args[i])?
                                    } else {
                                        Value::Null
                                    };
                                    method_scope.insert(param.clone(), arg_val);
                                }
                                
                                self.scopes.push(method_scope.clone());
                                let old_in_context = self.in_context;
                                self.in_context = true; // Set flag to indicate we're in a method
                                let mut result = Value::Null;
                                for stmt in body {
                                    if let Some(val) = self.execute_stmt(stmt)? {
                                        result = val;
                                        break;
                                    }
                                }
                                self.in_context = old_in_context; // Restore the flag
                                // Update object properties if they were modified
                                let updated_scope = self.scopes.pop().unwrap();
                                let mut updated_props = properties.clone();
                                for (name, val) in &updated_scope {
                                    if name != "this" && !params.contains(name) {
                                        updated_props.insert(name.clone(), val.clone());
                                    }
                                }
                                
                                // Update the object in scope if it came from a variable
                                if let Expr::Variable(var_name) = &**object {
                                    let updated_object = Value::Object {
                                        class_name: class_name.clone(),
                                        properties: updated_props,
                                    };
                                    self.set_variable(var_name.clone(), updated_object);
                                }
                                
                                Ok(result)
                            } else {
                                Err(format!("Method '{}' not found on class '{}'", method, class_name))
                            }
                        } else {
                            Err(format!("Class '{}' not found", class_name))
                        }
                    }
                    _ => Err(format!("Cannot call method on {}", obj_val.type_name())),
                }
            }
        }
    }

    fn literal_to_value(&self, lit: &Literal) -> Value {
        match lit {
            Literal::Number(n) => Value::Number(*n),
            Literal::String(s) => Value::String(s.clone()),
            Literal::Boolean(b) => Value::Boolean(*b),
            Literal::Null => Value::Null,
        }
    }

    fn apply_binary_op(&self, left: &Value, op: &BinaryOp, right: &Value) -> Result<Value, String> {
        match op {
            BinaryOp::Add => match (left, right) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
                (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
                _ => Err(format!("Cannot add {} and {}", left.type_name(), right.type_name())),
            },
            BinaryOp::Subtract => {
                let a = left.to_number()?;
                let b = right.to_number()?;
                Ok(Value::Number(a - b))
            }
            BinaryOp::Multiply => {
                let a = left.to_number()?;
                let b = right.to_number()?;
                Ok(Value::Number(a * b))
            }
            BinaryOp::Divide => {
                let a = left.to_number()?;
                let b = right.to_number()?;
                if b == 0.0 {
                    Err("Division by zero".to_string())
                } else {
                    Ok(Value::Number(a / b))
                }
            }
            BinaryOp::Equal => Ok(Value::Boolean(self.values_equal(left, right))),
            BinaryOp::NotEqual => Ok(Value::Boolean(!self.values_equal(left, right))),
            BinaryOp::Less => {
                let a = left.to_number()?;
                let b = right.to_number()?;
                Ok(Value::Boolean(a < b))
            }
            BinaryOp::LessEqual => {
                let a = left.to_number()?;
                let b = right.to_number()?;
                Ok(Value::Boolean(a <= b))
            }
            BinaryOp::Greater => {
                let a = left.to_number()?;
                let b = right.to_number()?;
                Ok(Value::Boolean(a > b))
            }
            BinaryOp::GreaterEqual => {
                let a = left.to_number()?;
                let b = right.to_number()?;
                Ok(Value::Boolean(a >= b))
            }
            BinaryOp::And => Ok(Value::Boolean(left.is_truthy() && right.is_truthy())),
            BinaryOp::Or => Ok(Value::Boolean(left.is_truthy() || right.is_truthy())),
        }
    }

    fn apply_unary_op(&self, op: &UnaryOp, val: &Value) -> Result<Value, String> {
        match op {
            UnaryOp::Not => Ok(Value::Boolean(!val.is_truthy())),
            UnaryOp::Negate => {
                let n = val.to_number()?;
                Ok(Value::Number(-n))
            }
        }
    }

    fn values_equal(&self, a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Number(x), Value::Number(y)) => x == y,
            (Value::String(x), Value::String(y)) => x == y,
            (Value::Boolean(x), Value::Boolean(y)) => x == y,
            (Value::Null, Value::Null) => true,
            _ => false,
        }
    }

    fn call_function(&mut self, name: &str, args: &[Expr]) -> Result<Value, String> {
        // Check if this is a private function and we're not in context
        if name.starts_with("_") && !self.in_context {
            return Err(format!("Cannot call private function '{}' from outside context", name));
        }

        // First check if it's a method call (first arg is the object)
        if name == "map" && !args.is_empty() {
            return self.call_map_method(args);
        }

        // Evaluate arguments
        let mut arg_values = Vec::new();
        for arg in args {
            arg_values.push(self.evaluate_expr(arg)?);
        }

        // Get function value
        let func = self.get_variable(name)?;

        match func {
            Value::Function { params, body, closure } => {
                if params.len() != arg_values.len() {
                    return Err(format!("Function {} expects {} arguments, got {}", name, params.len(), arg_values.len()));
                }

                self.push_scope();

                // Restore closure
                for (name, value) in closure {
                    self.define_variable(name, value);
                }

                // Bind parameters
                for (param, arg) in params.iter().zip(arg_values.iter()) {
                    self.define_variable(param.clone(), arg.clone());
                }

                // Execute body with context flag set
                let old_in_context = self.in_context;
                self.in_context = true;
                let mut result = Value::Null;
                for stmt in &body {
                    if let Some(val) = self.execute_stmt(stmt)? {
                        result = val;
                        break;
                    }
                }
                self.in_context = old_in_context;

                self.pop_scope();
                Ok(result)
            }
            Value::Lambda { params, body, closure } => {
                if params.len() != arg_values.len() {
                    return Err(format!("Lambda expects {} arguments, got {}", params.len(), arg_values.len()));
                }

                self.push_scope();

                // Restore closure
                for (name, value) in closure {
                    self.define_variable(name, value);
                }

                // Bind parameters
                for (param, arg) in params.iter().zip(arg_values.iter()) {
                    self.define_variable(param.clone(), arg.clone());
                }

                // Evaluate body with context flag set
                let old_in_context = self.in_context;
                self.in_context = true;
                let result = self.evaluate_expr(&body)?;
                self.in_context = old_in_context;

                self.pop_scope();
                Ok(result)
            }
            Value::NativeFunction { name, arity } => {
                if arity != arg_values.len() {
                    return Err(format!("Native function {} expects {} arguments, got {}", name, arity, arg_values.len()));
                }
                builtins::call_builtin(&name, arg_values)
            }
            _ => Err(format!("{} is not a function", name)),
        }
    }

    fn call_map_method(&mut self, args: &[Expr]) -> Result<Value, String> {
        if args.len() != 2 {
            return Err(format!("map expects 2 arguments (array, function), got {}", args.len()));
        }

        let array_val = self.evaluate_expr(&args[0])?;
        let func_val = self.evaluate_expr(&args[1])?;

        if let Value::Array(arr) = array_val {
            let mut result = Vec::new();
            
            for item in arr {
                match &func_val {
                    Value::Lambda { params, body, closure } => {
                        if params.len() != 1 {
                            return Err("map callback expects 1 parameter".to_string());
                        }

                        self.push_scope();

                        // Restore closure
                        for (name, value) in closure {
                            self.define_variable(name.clone(), value.clone());
                        }

                        // Bind parameter
                        self.define_variable(params[0].clone(), item);

                        // Evaluate body
                        let val = self.evaluate_expr(body)?;
                        result.push(val);

                        self.pop_scope();
                    }
                    _ => return Err("map expects a function as second argument".to_string()),
                }
            }

            Ok(Value::Array(result))
        } else {
            Err(format!("map expects an array, got {}", array_val.type_name()))
        }
    }

    fn match_value(&mut self, value: &Value, cases: &[MatchCase]) -> Result<Value, String> {
        for case in cases {
            if self.pattern_matches(&case.pattern, value)? {
                return self.evaluate_expr(&case.body);
            }
        }
        Err("No matching case found".to_string())
    }

    fn pattern_matches(&self, pattern: &Pattern, value: &Value) -> Result<bool, String> {
        match pattern {
            Pattern::Wildcard => Ok(true),
            Pattern::Literal(lit) => {
                let lit_val = self.literal_to_value(lit);
                Ok(self.values_equal(&lit_val, value))
            }
            Pattern::Identifier(id) => {
                // Match against type name
                Ok(id == value.type_name())
            }
        }
    }

    fn capture_closure(&self) -> HashMap<String, Value> {
        let mut closure = HashMap::new();
        
        // Capture all variables from current scopes
        for scope in &self.scopes {
            for (name, value) in scope {
                closure.insert(name.clone(), value.clone());
            }
        }

        closure
    }
}
