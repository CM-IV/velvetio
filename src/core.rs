// src/core.rs

use crate::{Parse, Result, VelvetIOError};
use std::io::{self, Write};

/// Holds the strongly-typed result of a form field
#[derive(Debug, Clone, PartialEq)]
pub enum FieldValue {
    Text(String),
    Number(f64),
    Boolean(bool),
    Choice(String),
    MultiChoice(Vec<String>),
    Optional(Option<String>),
    ValidatedText(String),
}

impl std::fmt::Display for FieldValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldValue::Text(s) | FieldValue::Choice(s) | FieldValue::ValidatedText(s) => {
                write!(f, "{}", s)
            }
            FieldValue::Number(n) => write!(f, "{}", n),
            FieldValue::Boolean(b) => write!(f, "{}", b),
            FieldValue::MultiChoice(v) => write!(f, "{}", v.join(", ")),
            FieldValue::Optional(o) => match o {
                Some(s) => write!(f, "{}", s),
                None => write!(f, ""),
            },
        }
    }
}

/// Prints the prompt and flushes stdout so it appears immediately.
fn print_prompt(text: &str) {
    print!("{}: ", text);
    let _ = io::stdout().flush();
}

/// Reads a line from stdin, returning a trimmed string.
/// Returns `Ok(Some(String))` on success, `Ok(None)` on EOF (Ctrl+D / closed pipe),
/// and `Err` on actual I/O errors.
fn read_line() -> io::Result<Option<String>> {
    let mut input = String::new();
    let bytes_read = io::stdin().read_line(&mut input)?;

    if bytes_read == 0 {
        Ok(None) // EOF encountered
    } else {
        Ok(Some(input.trim().to_string()))
    }
}

/// Keep asking until we get valid input
pub fn ask<T: Parse>(prompt: &str) -> T {
    loop {
        print_prompt(prompt);

        match read_line() {
            Ok(Some(input)) => match T::parse(&input) {
                Ok(value) => return value,
                Err(e) => eprintln!("{}", e),
            },
            Ok(None) => {
                eprintln!("\nUnexpected end of input (EOF). Exiting.");
                std::process::exit(1);
            }
            Err(e) => eprintln!("Input error: {}", e),
        }
    }
}

/// Try once, return Result instead of retrying
pub fn try_ask<T: Parse>(prompt: &str) -> Result<T> {
    print_prompt(prompt);

    match read_line()? {
        Some(input) => T::parse(&input),
        None => Err(VelvetIOError::new(
            "Unexpected end of input (EOF)",
            "",
            "valid input",
        )),
    }
}

/// Ask with validation function
pub fn ask_with_validation<T: Parse, F>(
    prompt: &str,
    validator: F,
    error_message: Option<&str>,
) -> T
where
    F: Fn(&T) -> bool,
{
    let error_msg = error_message.unwrap_or("Invalid input, please try again");

    loop {
        print_prompt(prompt);

        match read_line() {
            Ok(Some(input)) => match T::parse(&input) {
                Ok(value) => {
                    if validator(&value) {
                        return value;
                    } else {
                        eprintln!("{}", error_msg);
                    }
                }
                Err(e) => eprintln!("{}", e),
            },
            Ok(None) => {
                eprintln!("\nUnexpected end of input (EOF). Exiting.");
                std::process::exit(1);
            }
            Err(e) => eprintln!("Input error: {}", e),
        }
    }
}

/// Ask with default - hit enter to use default
pub fn ask_with_default<T: Parse + std::fmt::Display + Clone>(prompt: &str, default: T) -> T {
    print!("{} [{}]: ", prompt, default);
    let _ = io::stdout().flush();

    match read_line() {
        // Empty input (just Enter) -> use default
        Ok(Some(input)) if input.is_empty() => default,
        // Valid parse -> return value
        Ok(Some(input)) => match T::parse(&input) {
            Ok(value) => value,
            // Invalid parse -> warn user, then fall back to default
            Err(e) => {
                eprintln!("{}, using default: {}", e, default);
                default
            }
        },
        // EOF or IO error -> silently fall back to default
        _ => default,
    }
}

/// Yes/no question
pub fn confirm(prompt: &str) -> bool {
    ask::<bool>(&format!("{} (y/n)", prompt))
}

/// Pick one option from a list
pub fn choose<T>(prompt: &str, choices: &[T]) -> T
where
    T: std::fmt::Display + Clone,
{
    if choices.is_empty() {
        panic!("Cannot choose from empty list");
    }

    loop {
        println!("{}:", prompt);
        for (i, choice) in choices.iter().enumerate() {
            println!("  {}. {}", i + 1, choice);
        }

        match try_ask::<usize>(&format!("Choose (1-{})", choices.len())) {
            Ok(index) if index >= 1 && index <= choices.len() => {
                return choices[index - 1].clone();
            }
            Ok(_) => eprintln!("Please choose between 1 and {}", choices.len()),
            Err(e) => eprintln!("{}", e),
        }
    }
}

/// Pick multiple options from a list
pub fn multi_select<T>(prompt: &str, choices: &[T]) -> Vec<T>
where
    T: std::fmt::Display + Clone,
{
    if choices.is_empty() {
        return Vec::new();
    }

    loop {
        println!("{}:", prompt);
        for (i, choice) in choices.iter().enumerate() {
            println!("  {}. {}", i + 1, choice);
        }
        println!("Enter numbers separated by commas (e.g., 1,3,5) or 'all' or 'none':");

        let input = ask::<String>("Selection");
        let input = input.trim().to_lowercase();

        if input == "none" || input.is_empty() {
            return Vec::new();
        }

        if input == "all" {
            return choices.to_vec();
        }

        let parts: Vec<&str> = input.split(',').map(|s| s.trim()).collect();
        let mut selected = Vec::new();
        let mut valid = true;

        for part in parts {
            match part.parse::<usize>() {
                Ok(num) if num >= 1 && num <= choices.len() => {
                    selected.push(choices[num - 1].clone());
                }
                Ok(num) => {
                    eprintln!("{} is not a valid option (1-{})", num, choices.len());
                    valid = false;
                    break;
                }
                Err(_) => {
                    eprintln!("Please enter numbers separated by commas");
                    valid = false;
                    break;
                }
            }
        }

        if valid {
            return selected;
        }
    }
}

/// Form builder for collecting multiple inputs
pub struct Form {
    fields: Vec<FormField>,
}

struct FormField {
    key: String,
    prompt: String,
    field_type: FieldType,
}

enum FieldType {
    Text,
    Number,
    Boolean,
    Choice(Vec<String>),
    MultiChoice(Vec<String>),
    Optional,
    ValidatedText {
        validator: Box<dyn Fn(&str) -> bool>,
        error_msg: String,
    },
}

impl Form {
    pub fn new() -> Self {
        Self { fields: Vec::new() }
    }

    pub fn text(mut self, key: &str, prompt: &str) -> Self {
        self.fields.push(FormField {
            key: key.to_string(),
            prompt: prompt.to_string(),
            field_type: FieldType::Text,
        });
        self
    }

    pub fn number(mut self, key: &str, prompt: &str) -> Self {
        self.fields.push(FormField {
            key: key.to_string(),
            prompt: prompt.to_string(),
            field_type: FieldType::Number,
        });
        self
    }

    pub fn boolean(mut self, key: &str, prompt: &str) -> Self {
        self.fields.push(FormField {
            key: key.to_string(),
            prompt: prompt.to_string(),
            field_type: FieldType::Boolean,
        });
        self
    }

    pub fn choice(mut self, key: &str, prompt: &str, choices: &[&str]) -> Self {
        self.fields.push(FormField {
            key: key.to_string(),
            prompt: prompt.to_string(),
            field_type: FieldType::Choice(choices.iter().map(|s| s.to_string()).collect()),
        });
        self
    }

    pub fn multi_choice(mut self, key: &str, prompt: &str, choices: &[&str]) -> Self {
        self.fields.push(FormField {
            key: key.to_string(),
            prompt: prompt.to_string(),
            field_type: FieldType::MultiChoice(choices.iter().map(|s| s.to_string()).collect()),
        });
        self
    }

    pub fn optional(mut self, key: &str, prompt: &str) -> Self {
        self.fields.push(FormField {
            key: key.to_string(),
            prompt: format!("{} (optional)", prompt),
            field_type: FieldType::Optional,
        });
        self
    }

    pub fn validated_text<F>(
        mut self,
        key: &str,
        prompt: &str,
        validator: F,
        error_msg: &str,
    ) -> Self
    where
        F: Fn(&str) -> bool + 'static,
    {
        self.fields.push(FormField {
            key: key.to_string(),
            prompt: prompt.to_string(),
            field_type: FieldType::ValidatedText {
                validator: Box::new(validator),
                error_msg: error_msg.to_string(),
            },
        });
        self
    }

    /// Run through all fields and collect the results
    pub fn collect(self) -> std::collections::HashMap<String, FieldValue> {
        let mut results = std::collections::HashMap::new();
        for field in self.fields {
            let value = match field.field_type {
                FieldType::Text => FieldValue::Text(ask::<String>(&field.prompt)),
                FieldType::Number => FieldValue::Number(ask::<f64>(&field.prompt)),
                FieldType::Boolean => FieldValue::Boolean(ask::<bool>(&field.prompt)),
                FieldType::Choice(choices) => {
                    let choice_refs: Vec<&str> = choices.iter().map(|s| s.as_str()).collect();
                    FieldValue::Choice(choose(&field.prompt, &choice_refs).to_string())
                }
                FieldType::MultiChoice(choices) => {
                    let choice_refs: Vec<&str> = choices.iter().map(|s| s.as_str()).collect();
                    let selected = multi_select(&field.prompt, &choice_refs);
                    let selected_strings: Vec<String> =
                        selected.iter().map(|s| s.to_string()).collect();
                    FieldValue::MultiChoice(selected_strings)
                }
                FieldType::Optional => {
                    let input = ask::<String>(&field.prompt);
                    if input.trim().is_empty() {
                        FieldValue::Optional(None)
                    } else {
                        FieldValue::Optional(Some(input))
                    }
                }
                FieldType::ValidatedText {
                    validator,
                    error_msg,
                } => {
                    let res = ask_with_validation(
                        &field.prompt,
                        |s: &String| validator(s),
                        Some(&error_msg),
                    );
                    FieldValue::ValidatedText(res)
                }
            };
            results.insert(field.key, value);
        }
        results
    }
}

pub fn form() -> Form {
    Form::new()
}
