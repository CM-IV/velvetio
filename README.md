# VelvetIO

CLI input for Rust that doesn't suck.

Tired of wrestling with `stdin().read_line()` and manual parsing? VelvetIO handles the annoying stuff so you can focus on building your CLI tool.

```rust
use velvetio::prelude::*;

let name = ask!("Your name");
let age = ask!("Age" => u32);

if confirm!("Continue?") {
    println!("Hello {}, age {}", name, age);
}
```

## Installation

```toml
[dependencies]
velvetio = "0.1"
```

## Features

- **Actually zero dependencies** - No bloat, fast builds
- **Type-safe parsing** - Works with any type that makes sense
- **Smart validation** - Built-in validators + custom functions  
- **Form builder** - Collect multiple inputs without repetition
- **Helpful errors** - Clear messages when things go wrong
- **Flexible** - Start simple, add complexity as needed

## Quick Examples

### Basic Input

```rust
use velvetio::prelude::*;

// Strings (most common)
let name = ask!("Name");

// Numbers
let port = ask!("Port" => u16);
let price = ask!("Price" => f64);

// Booleans (accepts y/n, yes/no, true/false, 1/0)
let enabled = ask!("Enable feature?" => bool);
```

### Input with Defaults

```rust
// Hit enter to use the default
let host = ask!("Host", default: "localhost".to_string());
let port = ask!("Port" => u16, default: 8080);

// Try once, fall back if parsing fails
let timeout = ask!("Timeout" => u32, or: 30);
```

### Validation

```rust
// Simple validation
let email = ask!("Email", validate: |s| s.contains('@'));

// With custom error message
let username = ask!(
    "Username",
    validate: |s: &String| s.len() >= 3,
    error: "Username must be at least 3 characters"
);

// Built-in validators
let password = ask!(
    "Password",
    validate: and(min_length(8), not_empty),
    error: "Password must be at least 8 characters"
);
```

### Choice Selection

```rust
// Pick one
let os = choose!("Operating System", [
    "Linux",
    "macOS", 
    "Windows"
]);

// Pick multiple (comma-separated: 1,3,5 or "all" or "none")
let features = multi_select!("Features to enable", [
    "Authentication",
    "Logging",
    "Caching",
    "Metrics"
]);
```

### Yes/No Questions

```rust
let proceed = confirm!("Delete all files?");
let save_config = confirm!("Save configuration?");
```

## Form Builder

For collecting multiple related inputs:

```rust
let config = form()
    .text("app_name", "Application name")
    .number("port", "Port number")
    .boolean("debug", "Enable debug mode?")
    .choice("env", "Environment", &["dev", "staging", "prod"])
    .multi_choice("features", "Features", &["auth", "db", "cache"])
    .optional("description", "Description (optional)")
    .validated_text(
        "email", 
        "Admin email",
        |email| email.contains('@'),
        "Must be a valid email"
    )
    .collect();

// Access values
let app_name = config.get("app_name").unwrap();
let port: u16 = config.get("port").unwrap().parse().unwrap();
```

### Quick Forms

For simple cases:

```rust
let info = quick_form! {
    "name" => "Your name",
    "email" => "Email address",
    "company" => "Company"
};
```

## Advanced Type Parsing

VelvetIO parses many types automatically:

### Collections

```rust
// Vec - detects separators automatically
let numbers: Vec<u32> = ask!("Numbers" => Vec<u32>);
// Input: "1,2,3" or "1 2 3" or "1;2;3" or "1|2|3"

let tags: Vec<String> = ask!("Tags" => Vec<String>);
// Input: "rust,cli,tool"
```

### Tuples

```rust
// Pairs
let coords: (f64, f64) = ask!("Coordinates (lat,lng)" => (f64, f64));
// Input: "40.7,-74.0" or "40.7 -74.0"

// Triples
let rgb: (u8, u8, u8) = ask!("RGB color" => (u8, u8, u8));
// Input: "255,128,0"
```

### Optional Values

```rust
let backup_email: Option<String> = ask!("Backup email" => Option<String>);
// Empty input, "none", "null", "-", or "skip" becomes None
// Anything else gets parsed as Some(value)
```

## Custom Types

Make your own types work with VelvetIO:

```rust
#[derive(Debug)]
struct UserId(u32);

impl std::str::FromStr for UserId {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()> {
        s.parse::<u32>().map(UserId).map_err(|_| ())
    }
}

// Add parsing support
quick_parse!(UserId);

// Now you can use it
let user_id = ask!("User ID" => UserId);
```

## Validation

### Built-in Validators

```rust
// String validators
not_empty           // String is not empty
min_length(n)       // At least n characters
max_length(n)       // At most n characters

// Number validators  
is_positive         // Greater than zero
in_range(min, max)  // Between min and max (inclusive)

// Combining validators
and(v1, v2)         // Both must pass
or(v1, v2)          // Either can pass
```

### Custom Validators

```rust
// Email validation
let email = ask!(
    "Email",
    validate: |s: &String| s.contains('@') && s.contains('.'),
    error: "Please enter a valid email"
);

// Port range
let port = ask!(
    "Port" => u16,
    validate: in_range(1024, 65535),
    error: "Port must be between 1024 and 65535"
);

// Complex validation
let username = ask!(
    "Username",
    validate: and(
        min_length(3),
        |s: &String| s.chars().all(|c| c.is_alphanumeric() || c == '_')
    ),
    error: "Username must be 3+ chars, alphanumeric and underscores only"
);
```

## Error Handling

Most functions retry automatically on invalid input. Use `try_ask!` if you want to handle errors yourself:

```rust
match try_ask!("Age" => u32) {
    Ok(age) => println!("Age: {}", age),
    Err(e) => {
        eprintln!("Failed to get age: {}", e);
        std::process::exit(1);
    }
}
```

## Boolean Parsing

Accepts many formats:

| Input | Result |
|-------|--------|
| `y`, `yes`, `true`, `t`, `1`, `on` | `true` |
| `n`, `no`, `false`, `f`, `0`, `off` | `false` |

Case insensitive.

## Important Notes

### Passwords

VelvetIO doesn't include password input to keep zero dependencies. For secure password input, use the `rpassword` crate:

```toml
[dependencies]
velvetio = "0.1"
rpassword = "7.0"
```

```rust
use velvetio::prelude::*;

let username = ask!("Username");
let password = rpassword::prompt_password("Password: ").unwrap();
```

### Performance

- Input is read synchronously (blocks until user responds)
- Form building is cheap - only prompts when you call `.collect()`
- Vec parsing pre-allocates based on detected items
- No heap allocations for simple types

### Thread Safety

VelvetIO uses `stdin()`/`stdout()` which are globally shared. Don't use from multiple threads simultaneously.

## Examples

Check out `examples/setup_wizard.rs` for a comprehensive demo:

```bash
cargo run --example setup_wizard
```

## Comparison with Other Crates

| Feature | VelvetIO | dialoguer | inquire |
|---------|----------|-----------|---------|
| Dependencies | 0 | 3+ | 5+ |
| Forms | ✅ Elegant | ❌ None | ❌ None |
| API | `ask!("Name")` | Verbose builders | Complex setup |
| Type parsing | ✅ Automatic | ❌ Manual | ❌ Manual |
| Maintenance | ✅ Active | ⚠️ Stagnant | ⚠️ Slow |

## Common Patterns

### Configuration Wizard

```rust
let config = form()
    .text("name", "Project name")
    .choice("framework", "Framework", &["Axum", "Warp", "Rocket"])
    .multi_choice("features", "Features", &["Auth", "DB", "Cache"])
    .boolean("docker", "Use Docker?")
    .collect();
```

### User Registration

```rust
let username = ask!(
    "Username",
    validate: and(min_length(3), max_length(20)),
    error: "Username must be 3-20 characters"
);

let email = ask!(
    "Email", 
    validate: |s: &String| s.contains('@'),
    error: "Please enter a valid email"
);

let age: Option<u32> = ask!("Age (optional)" => Option<u32>);
```

### Server Configuration

```rust
let host = ask!("Host", default: "0.0.0.0".to_string());
let port = ask!("Port" => u16, default: 8080);
let workers = ask!("Worker threads" => usize, default: 4);

let ssl = confirm!("Enable SSL?");
if ssl {
    let cert_path = ask!("Certificate path");
    let key_path = ask!("Private key path");
}
```

## Contributing

Found a bug or want to add a feature? PRs welcome!

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.