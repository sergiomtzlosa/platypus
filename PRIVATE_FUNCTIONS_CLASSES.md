# Private Functions and Classes Implementation

## Overview
Extended the Platypus language privacy system to support private functions and private classes, in addition to the existing private variables. All use the underscore (`_`) prefix naming convention.

## Features

### 1. Private Functions
- Functions declared with `_` prefix can only be called from within other functions or methods
- Attempting to call a private function from global scope results in an error
- Private functions are useful for internal helper functions and implementation details

### 2. Private Classes  
- Classes declared with `_` prefix can only be instantiated from within functions or methods
- Attempting to instantiate a private class from global scope results in an error
- Private classes are useful for internal implementation classes not meant for external use

### 3. Private Variables (previously implemented)
- Properties declared with `_` prefix on class instances
- Only accessible from within class methods

## Implementation Details

### Context Tracking
- Renamed `in_method` field to `in_context` for broader applicability
- `in_context` is set to `true` when executing:
  - Function bodies
  - Method bodies
  - Lambda expressions

### Privacy Checks

**FunctionCall Expression**
- Before calling any function, check if its name starts with `_`
- If private and `in_context` is false, return error
- Set `in_context = true` before executing function body

**New Expression**
- Before instantiating any class, check if its name starts with `_`
- If private and `in_context` is false, return error

**PropertyAccess/PropertyAssign**
- Existing checks for private variables with `_` prefix
- Uses `in_context` flag to allow access from within methods

## Examples Created

### 1. `private_functions.plat`
Demonstrates:
- Declaring and calling private functions from within public functions
- Error when attempting to call private functions from global scope

### 2. `private_classes.plat`
Demonstrates:
- Declaring and instantiating private classes from within public classes
- Declaring and instantiating private classes from within functions
- Error when attempting to instantiate private classes from global scope

### 3. `private_comprehensive.plat`
Demonstrates all three privacy features working together:
- Private variables in a public class
- Private functions used by public methods
- Private classes instantiated within public class methods

## Usage Examples

```platypus
// Private function - only callable from other functions
func _internalHelper(x) {
    return x * 2
}

func publicFunction(x) {
    // Can call private functions from within functions
    return _internalHelper(x)
}

// Private class - only instantiable from functions/methods
class _InternalBuffer {
    _data = 0
}

class PublicAPI {
    func usePrivate() {
        // Can instantiate private classes from within class methods
        buffer = new _InternalBuffer()
    }
}

// These work:
publicFunction(5)              // ✓ calls _internalHelper
api = new PublicAPI()          // ✓ instantiates public class
api.usePrivate()               // ✓ instantiates _InternalBuffer

// These fail:
_internalHelper(5)             // ✗ Error: private function from global scope
buffer = new _InternalBuffer() // ✗ Error: private class from global scope
```

## Test Results

All 14 examples pass:
- ✓ hello.plat
- ✓ functions.plat  
- ✓ pattern_matching.plat
- ✓ control_flow.plat
- ✓ loops.plat
- ✓ for_foreach.plat
- ✓ showcase.plat
- ✓ classes.plat
- ✓ private_variables.plat
- ✓ method_property_modification.plat
- ✓ bank_account.plat
- ✓ private_functions.plat (NEW)
- ✓ private_classes.plat (NEW)
- ✓ private_comprehensive.plat (NEW)

## Encapsulation Hierarchy

The privacy system now provides three levels of encapsulation:

1. **Class-level privacy** - Private properties within class instances
   - Format: `_propertyName`
   - Scope: Accessible only to methods of that class

2. **Module-level privacy** - Private functions at global scope
   - Format: `_functionName`
   - Scope: Callable only from within functions/methods

3. **Type privacy** - Private classes at global scope
   - Format: `_ClassName`
   - Scope: Instantiable only from within functions/methods

This creates a comprehensive privacy/encapsulation system suitable for building modular, well-organized code.
