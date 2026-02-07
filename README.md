# Platypus Programming Language

![Platypus Logo](https://img.shields.io/badge/Platypus-v0.1.0-blue)
![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)
![License](https://img.shields.io/badge/license-MIT-green)

**Platypus** is an compiled programming language that blends a mixture of features from Python, JavaScript, and Scala. Written in Rust.

## ✨ Key Features

- **🚀 Compiled Language**: Benefit from static analysis and optimized execution
- **🎯 Dynamic Type System**: Write code without explicit type annotations, like Python
- **📝 Python-like Syntax**: Clear and concise variable declarations
- **🎨 Mixed Syntax**: Combines JavaScript and Scala idioms for expressiveness
- **🔥 Pattern Matching**: Powerful match expressions for control flow
- **⚡ Higher-Order Functions**: First-class functions and lambdas

## 📦 Installation

### Prerequisites
- Rust 1.70 or higher
- Cargo (comes with Rust)

### Build from Source

```bash
# Clone the repository
git clone https://github.com/sergiomtzlosa/platypus
cd platypus

# Build in release mode
cargo build --release

# The binary will be at target/release/platypus
```

### Add to PATH (Optional)

```bash
# For macOS/Linux
export PATH="$PATH:$(pwd)/target/release"

# Or create a symlink
sudo ln -s $(pwd)/target/release/platypus /usr/local/bin/platypus
```

## 🚀 Quick Start

### Running Your First Program

Create a file `hello.plat`:

```platypus
greeting = "Hello, Platypus!"
print(greeting)

count = 42
print(count)
```

Run it:

```bash
platypus run hello.plat
```

Output:
```
Hello, Platypus!
42
```

### Interactive REPL

Start the REPL for interactive coding:

```bash
platypus repl
```

```
Platypus REPL v0.1.0
Type 'exit' or press Ctrl+D to quit

>> x = 42
>> x + 10
52
>> greeting = "Hello!"
>> print(greeting)
Hello!
>> exit
Goodbye!
```

## 📚 Language Syntax

### Variables

Variables are dynamically typed—no type declarations needed:

```platypus
name = "Alice"
age = 30
pi = 3.14159
isActive = true
items = [1, 2, 3, 4, 5]
nothing = null
```

### Functions

Define functions with optional type annotations:

```platypus
// Function without type annotation
func greet(name) {
    return "Hello, " + name + "!"
}

// Function with return type annotation
func add(a, b): Number {
    return a + b
}

// Call functions
result = add(10, 32)
print(result)  // 42

message = greet("World")
print(message)  // Hello, World!
```

### Higher-Order Functions and Lambdas

Functions are first-class citizens:

```platypus
numbers = [1, 2, 3, 4, 5]

// Using lambda with map
doubled = numbers.map(n => n * 2)
print(doubled)  // [2, 4, 6, 8, 10]

squared = numbers.map(x => x * x)
print(squared)  // [1, 4, 9, 16, 25]
```

### Pattern Matching

Powerful match expressions for control flow:

```platypus
func describe(value): String {
    result = match (typeof(value)) {
        case "String" => "It's a string!"
        case "Number" => "It's a number!"
        case "Boolean" => "It's a boolean!"
        case "Array" => "It's an array!"
        case _ => "Unknown type!"
    }
    return result
}

print(describe("Hello"))    // It's a string!
print(describe(42))         // It's a number!
print(describe([1, 2, 3]))  // It's an array!
```

### Control Flow

#### If-Else Statements

```platypus
age = 25
if (age >= 18) {
    print("Adult")
} else {
    print("Minor")
}
```

#### While Loops

```platypus
counter = 0
while (counter < 5) {
    print(counter)
    counter = counter + 1
}
```

### Built-in Functions

Platypus provides several built-in functions:

- **`print(value)`**: Print a value to stdout
- **`typeof(value)`**: Returns the type of a value as a string
- **`len(array_or_string)`**: Returns the length of an array or string
- **`map(array, function)`**: Apply a function to each element (method syntax: `array.map(fn)`)

## 📖 Example Programs

### Control Flow Example

See [`examples/control_flow.plat`](examples/control_flow.plat) for comprehensive control structure examples including:
- If-then-else statements
- While loops
- Simulated for and foreach loops
- Nested loops

Run it:
```bash
platypus run examples/control_flow.plat
```

### Pattern Matching Example

See [`examples/pattern_matching.plat`](examples/pattern_matching.plat):

```bash
platypus run examples/pattern_matching.plat
```

### Functions and Higher-Order Functions

See [`examples/functions.plat`](examples/functions.plat):

```bash
platypus run examples/functions.plat
```

### Complete Showcase

See [`examples/showcase.plat`](examples/showcase.plat) for a comprehensive feature demonstration:

```bash
platypus run examples/showcase.plat
```

## 🛠️ Compiling and Running Programs

### Compile and Run

```bash
# Run a Platypus source file
platypus run <file.plat>

# Example
platypus run examples/hello.plat
```

### Interactive REPL

```bash
# Start the REPL
platypus repl
```

### Help and Version

```bash
# Show help
platypus --help

# Show version
platypus --version
```

## 📋 Language Specification

### Data Types

- **Number**: 64-bit floating-point (`42`, `3.14`)
- **String**: UTF-8 strings (`"Hello"`)
- **Boolean**: `true` or `false`
- **Array**: Homogeneous or heterogeneous collections (`[1, 2, 3]`)
- **Function**: First-class functions and lambdas
- **Null**: Represents absence of value

### Operators

**Arithmetic**: `+`, `-`, `*`, `/`  
**Comparison**: `==`, `!=`, `<`, `>`, `<=`, `>=`  
**Logical**: `&&`, `||`, `!`  
**Assignment**: `=`

### Type Coercion

Numbers, strings, and booleans can be used in arithmetic operations with automatic coercion where sensible.

## 🎯 Project Structure

```
platypus/
├── Cargo.toml              # Rust package configuration
├── src/
│   ├── main.rs             # CLI entry point
│   ├── lexer/
│   │   ├── mod.rs          # Tokenizer
│   │   └── token.rs        # Token types
│   ├── parser/
│   │   ├── mod.rs          # Parser
│   │   └── ast.rs          # AST definitions
│   └── runtime/
│       ├── mod.rs          # Interpreter
│       ├── value.rs        # Runtime values
│       └── builtins.rs     # Built-in functions
└── examples/
    ├── hello.plat          # Hello world
    ├── functions.plat      # Function examples
    ├── pattern_matching.plat
    ├── control_flow.plat   # Control structures
    ├── design_patterns.plat # OOP patterns
    └── showcase.plat       # Complete feature demo
```

## 🧪 Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

## 📄 License

MIT License - feel free to use Platypus in your projects!

## 🚧 Future Roadmap

- [ ] Array indexing (`arr[0]`)
- [ ] For and foreach loop syntax
- [ ] Object/class system
- [ ] Import/module system
- [ ] Native code generation (LLVM backend)
- [ ] Standard library expansion
- [ ] Package manager

## 📞 Support

For questions, issues, or feature requests, please open an issue on the repository.

---

**Happy coding with Platypus! 🦆**
