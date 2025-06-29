//! Flag system for command-line argument parsing
//!
//! This module provides a flexible flag parsing system that supports:
//! - Multiple value types (string, bool, int, float, string slice)
//! - Short and long flag names
//! - Required and optional flags
//! - Default values
//! - Hierarchical flag inheritance from parent commands

use crate::completion::{CompletionFunc, CompletionResult};
use crate::error::{Error, Result};
use std::collections::HashSet;

/// Represents the value of a parsed flag
///
/// `FlagValue` is an enum that can hold different types of values
/// that flags can have. This allows for type-safe access to flag values.
///
/// # Examples
///
/// ```
/// use flag_rs::flag::{FlagValue, FlagType, Flag};
///
/// // Parse different types of values
/// let string_flag = Flag::new("name").value_type(FlagType::String);
/// let value = string_flag.parse_value("John").unwrap();
/// assert_eq!(value.as_string().unwrap(), "John");
///
/// let bool_flag = Flag::new("verbose").value_type(FlagType::Bool);
/// let value = bool_flag.parse_value("true").unwrap();
/// assert!(value.as_bool().unwrap());
/// ```
#[derive(Clone, Debug, PartialEq)]
pub enum FlagValue {
    /// A string value
    String(String),
    /// A boolean value
    Bool(bool),
    /// An integer value
    Int(i64),
    /// A floating-point value
    Float(f64),
    /// A slice of strings (for repeated flags)
    StringSlice(Vec<String>),
}

impl FlagValue {
    /// Returns the value as a string reference
    ///
    /// # Errors
    ///
    /// Returns `Error::FlagParsing` if the value is not a string
    ///
    /// # Examples
    ///
    /// ```
    /// use flag_rs::flag::FlagValue;
    ///
    /// let value = FlagValue::String("hello".to_string());
    /// assert_eq!(value.as_string().unwrap(), "hello");
    ///
    /// let value = FlagValue::Bool(true);
    /// assert!(value.as_string().is_err());
    /// ```
    pub fn as_string(&self) -> Result<&String> {
        match self {
            Self::String(s) => Ok(s),
            _ => Err(Error::flag_parsing("Flag value is not a string")),
        }
    }

    /// Returns the value as a boolean
    ///
    /// # Errors
    ///
    /// Returns `Error::FlagParsing` if the value is not a boolean
    pub fn as_bool(&self) -> Result<bool> {
        match self {
            Self::Bool(b) => Ok(*b),
            _ => Err(Error::flag_parsing("Flag value is not a boolean")),
        }
    }

    /// Returns the value as an integer
    ///
    /// # Errors
    ///
    /// Returns `Error::FlagParsing` if the value is not an integer
    pub fn as_int(&self) -> Result<i64> {
        match self {
            Self::Int(i) => Ok(*i),
            _ => Err(Error::flag_parsing("Flag value is not an integer")),
        }
    }

    /// Returns the value as a float
    ///
    /// # Errors
    ///
    /// Returns `Error::FlagParsing` if the value is not a float
    pub fn as_float(&self) -> Result<f64> {
        match self {
            Self::Float(f) => Ok(*f),
            _ => Err(Error::flag_parsing("Flag value is not a float")),
        }
    }

    /// Returns the value as a string slice reference
    ///
    /// # Errors
    ///
    /// Returns `Error::FlagParsing` if the value is not a string slice
    pub fn as_string_slice(&self) -> Result<&Vec<String>> {
        match self {
            Self::StringSlice(v) => Ok(v),
            _ => Err(Error::flag_parsing("Flag value is not a string slice")),
        }
    }
}

/// Represents constraints that can be applied to flags
///
/// Flag constraints allow you to define relationships between flags,
/// such as mutual exclusivity or dependencies.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FlagConstraint {
    /// This flag is required if another flag is set
    RequiredIf(String),
    /// This flag conflicts with other flags (mutually exclusive)
    ConflictsWith(Vec<String>),
    /// This flag requires other flags to be set
    Requires(Vec<String>),
}

/// Represents a command-line flag
///
/// A `Flag` defines a command-line option that can be passed to a command.
/// Flags can have both long names (e.g., `--verbose`) and short names (e.g., `-v`).
///
/// # Examples
///
/// ```
/// use flag_rs::flag::{Flag, FlagType, FlagValue};
///
/// // Create a boolean flag
/// let verbose = Flag::new("verbose")
///     .short('v')
///     .usage("Enable verbose output")
///     .value_type(FlagType::Bool)
///     .default(FlagValue::Bool(false));
///
/// // Create a string flag with validation
/// let name = Flag::new("name")
///     .short('n')
///     .usage("Name of the resource")
///     .value_type(FlagType::String)
///     .required();
/// ```
pub struct Flag {
    /// The long name of the flag (e.g., "verbose" for --verbose)
    pub name: String,
    /// The optional short name of the flag (e.g., 'v' for -v)
    pub short: Option<char>,
    /// A description of what the flag does
    pub usage: String,
    /// The default value if the flag is not provided
    pub default: Option<FlagValue>,
    /// Whether this flag must be provided
    pub required: bool,
    /// The type of value this flag accepts
    pub value_type: FlagType,
    /// Constraints applied to this flag
    pub constraints: Vec<FlagConstraint>,
    /// Optional completion function for this flag's values
    pub completion: Option<CompletionFunc>,
}

/// Represents the type of value a flag accepts
///
/// This enum determines how flag values are parsed from string input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FlagType {
    /// Accepts any string value
    String,
    /// Accepts boolean values (true/false, yes/no, 1/0)
    Bool,
    /// Accepts integer values
    Int,
    /// Accepts floating-point values
    Float,
    /// Accepts multiple string values (can be specified multiple times)
    StringSlice,
    /// Accepts multiple string values with accumulation (--tag=a --tag=b)
    StringArray,
    /// Must be one of a predefined set of values
    Choice(Vec<String>),
    /// Numeric value within a specific range
    Range(i64, i64),
    /// Must be a valid file path
    File,
    /// Must be a valid directory path
    Directory,
}

impl Flag {
    /// Creates a new flag with the given name
    ///
    /// # Examples
    ///
    /// ```
    /// use flag_rs::flag::Flag;
    ///
    /// let flag = Flag::new("verbose");
    /// assert_eq!(flag.name, "verbose");
    /// ```
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            short: None,
            usage: String::new(),
            default: None,
            required: false,
            value_type: FlagType::String,
            constraints: Vec::new(),
            completion: None,
        }
    }

    /// Creates a new boolean flag
    ///
    /// # Examples
    ///
    /// ```
    /// use flag_rs::flag::Flag;
    ///
    /// let flag = Flag::bool("verbose");
    /// ```
    #[must_use]
    pub fn bool(name: impl Into<String>) -> Self {
        Self::new(name).value_type(FlagType::Bool)
    }

    /// Creates a new integer flag
    ///
    /// # Examples
    ///
    /// ```
    /// use flag_rs::flag::Flag;
    ///
    /// let flag = Flag::int("port");
    /// ```
    #[must_use]
    pub fn int(name: impl Into<String>) -> Self {
        Self::new(name).value_type(FlagType::Int)
    }

    /// Creates a new float flag
    ///
    /// # Examples
    ///
    /// ```
    /// use flag_rs::flag::Flag;
    ///
    /// let flag = Flag::float("ratio");
    /// ```
    #[must_use]
    pub fn float(name: impl Into<String>) -> Self {
        Self::new(name).value_type(FlagType::Float)
    }

    /// Creates a new string flag
    ///
    /// # Examples
    ///
    /// ```
    /// use flag_rs::flag::Flag;
    ///
    /// let flag = Flag::string("name");
    /// ```
    #[must_use]
    pub fn string(name: impl Into<String>) -> Self {
        Self::new(name) // String is the default type
    }

    /// Creates a new string slice flag (can be specified multiple times)
    ///
    /// # Examples
    ///
    /// ```
    /// use flag_rs::flag::Flag;
    ///
    /// let flag = Flag::string_slice("tag");
    /// ```
    #[must_use]
    pub fn string_slice(name: impl Into<String>) -> Self {
        Self::new(name).value_type(FlagType::StringSlice)
    }

    /// Creates a new choice flag with allowed values
    ///
    /// # Examples
    ///
    /// ```
    /// use flag_rs::flag::Flag;
    ///
    /// let flag = Flag::choice("format", &["json", "yaml", "xml"]);
    /// ```
    #[must_use]
    pub fn choice(name: impl Into<String>, choices: &[&str]) -> Self {
        let choices: Vec<String> = choices.iter().map(|&s| s.to_string()).collect();
        Self::new(name).value_type(FlagType::Choice(choices))
    }

    /// Creates a new range flag with min and max values
    ///
    /// # Examples
    ///
    /// ```
    /// use flag_rs::flag::Flag;
    ///
    /// let flag = Flag::range("workers", 1, 16);
    /// ```
    #[must_use]
    pub fn range(name: impl Into<String>, min: i64, max: i64) -> Self {
        Self::new(name).value_type(FlagType::Range(min, max))
    }

    /// Creates a new file flag
    ///
    /// # Examples
    ///
    /// ```
    /// use flag_rs::flag::Flag;
    ///
    /// let flag = Flag::file("config");
    /// ```
    #[must_use]
    pub fn file(name: impl Into<String>) -> Self {
        Self::new(name).value_type(FlagType::File)
    }

    /// Creates a new directory flag
    ///
    /// # Examples
    ///
    /// ```
    /// use flag_rs::flag::Flag;
    ///
    /// let flag = Flag::directory("output");
    /// ```
    #[must_use]
    pub fn directory(name: impl Into<String>) -> Self {
        Self::new(name).value_type(FlagType::Directory)
    }

    /// Sets the short name for this flag
    ///
    /// # Examples
    ///
    /// ```
    /// use flag_rs::flag::Flag;
    ///
    /// let flag = Flag::new("verbose").short('v');
    /// assert_eq!(flag.short, Some('v'));
    /// ```
    #[must_use]
    pub const fn short(mut self, short: char) -> Self {
        self.short = Some(short);
        self
    }

    /// Sets the usage description for this flag
    ///
    /// # Examples
    ///
    /// ```
    /// use flag_rs::flag::Flag;
    ///
    /// let flag = Flag::new("verbose").usage("Enable verbose output");
    /// assert_eq!(flag.usage, "Enable verbose output");
    /// ```
    #[must_use]
    pub fn usage(mut self, usage: impl Into<String>) -> Self {
        self.usage = usage.into();
        self
    }

    /// Sets the default value for this flag
    ///
    /// # Examples
    ///
    /// ```
    /// use flag_rs::flag::{Flag, FlagValue};
    ///
    /// let flag = Flag::new("count").default(FlagValue::Int(10));
    /// assert_eq!(flag.default, Some(FlagValue::Int(10)));
    /// ```
    #[must_use]
    pub fn default(mut self, value: FlagValue) -> Self {
        self.default = Some(value);
        self
    }

    /// Sets a default boolean value
    ///
    /// # Examples
    ///
    /// ```
    /// use flag_rs::flag::{Flag, FlagValue};
    ///
    /// let flag = Flag::bool("verbose").default_bool(true);
    /// assert_eq!(flag.default, Some(FlagValue::Bool(true)));
    /// ```
    #[must_use]
    pub fn default_bool(self, value: bool) -> Self {
        self.default(FlagValue::Bool(value))
    }

    /// Sets a default string value
    ///
    /// # Examples
    ///
    /// ```
    /// use flag_rs::flag::{Flag, FlagValue};
    ///
    /// let flag = Flag::string("name").default_str("anonymous");
    /// assert_eq!(flag.default, Some(FlagValue::String("anonymous".to_string())));
    /// ```
    #[must_use]
    pub fn default_str(self, value: &str) -> Self {
        self.default(FlagValue::String(value.to_string()))
    }

    /// Sets a default integer value
    ///
    /// # Examples
    ///
    /// ```
    /// use flag_rs::flag::{Flag, FlagValue};
    ///
    /// let flag = Flag::int("port").default_int(8080);
    /// assert_eq!(flag.default, Some(FlagValue::Int(8080)));
    /// ```
    #[must_use]
    pub fn default_int(self, value: i64) -> Self {
        self.default(FlagValue::Int(value))
    }

    /// Sets a default float value
    ///
    /// # Examples
    ///
    /// ```
    /// use flag_rs::flag::{Flag, FlagValue};
    ///
    /// let flag = Flag::float("ratio").default_float(0.5);
    /// assert_eq!(flag.default, Some(FlagValue::Float(0.5)));
    /// ```
    #[must_use]
    pub fn default_float(self, value: f64) -> Self {
        self.default(FlagValue::Float(value))
    }

    /// Marks this flag as required
    ///
    /// # Examples
    ///
    /// ```
    /// use flag_rs::flag::Flag;
    ///
    /// let flag = Flag::new("name").required();
    /// assert!(flag.required);
    /// ```
    #[must_use]
    pub const fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Sets the value type for this flag
    ///
    /// # Examples
    ///
    /// ```
    /// use flag_rs::flag::{Flag, FlagType};
    ///
    /// let flag = Flag::new("count").value_type(FlagType::Int);
    /// ```
    #[must_use]
    pub fn value_type(mut self, value_type: FlagType) -> Self {
        self.value_type = value_type;
        self
    }

    /// Adds a constraint to this flag
    ///
    /// # Examples
    ///
    /// ```
    /// use flag_rs::flag::{Flag, FlagConstraint};
    ///
    /// let flag = Flag::new("ssl")
    ///     .constraint(FlagConstraint::RequiredIf("port".to_string()))
    ///     .constraint(FlagConstraint::ConflictsWith(vec!["no-ssl".to_string()]));
    /// ```
    #[must_use]
    pub fn constraint(mut self, constraint: FlagConstraint) -> Self {
        self.constraints.push(constraint);
        self
    }

    /// Sets a completion function for this flag's values
    ///
    /// # Arguments
    ///
    /// * `completion` - A function that generates completions for flag values
    ///
    /// # Examples
    ///
    /// ```
    /// use flag_rs::flag::Flag;
    /// use flag_rs::completion::CompletionResult;
    ///
    /// let flag = Flag::file("config")
    ///     .completion(|ctx, prefix| {
    ///         // In a real application, you might list config files
    ///         let configs = vec!["default.conf", "production.conf", "test.conf"];
    ///         Ok(CompletionResult::new().extend(
    ///             configs.into_iter()
    ///                 .filter(|c| c.starts_with(prefix))
    ///                 .map(String::from)
    ///         ))
    ///     });
    /// ```
    #[must_use]
    pub fn completion<F>(mut self, completion: F) -> Self
    where
        F: Fn(&crate::Context, &str) -> Result<CompletionResult> + Send + Sync + 'static,
    {
        self.completion = Some(Box::new(completion));
        self
    }
}

impl Clone for Flag {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            short: self.short,
            usage: self.usage.clone(),
            default: self.default.clone(),
            required: self.required,
            value_type: self.value_type.clone(),
            constraints: self.constraints.clone(),
            completion: None, // Don't clone the completion function
        }
    }
}

impl Flag {
    /// Parses a string value according to this flag's type
    ///
    /// # Arguments
    ///
    /// * `input` - The string value to parse
    ///
    /// # Returns
    ///
    /// Returns the parsed `FlagValue` on success
    ///
    /// # Errors
    ///
    /// Returns `Error::FlagParsing` if the input cannot be parsed as the expected type
    ///
    /// # Examples
    ///
    /// ```
    /// use flag_rs::flag::{Flag, FlagType, FlagValue};
    ///
    /// let int_flag = Flag::new("count").value_type(FlagType::Int);
    /// match int_flag.parse_value("42") {
    ///     Ok(FlagValue::Int(n)) => assert_eq!(n, 42),
    ///     _ => panic!("Expected Int value"),
    /// }
    ///
    /// let bool_flag = Flag::new("verbose").value_type(FlagType::Bool);
    /// match bool_flag.parse_value("true") {
    ///     Ok(FlagValue::Bool(b)) => assert!(b),
    ///     _ => panic!("Expected Bool value"),
    /// }
    /// ```
    pub fn parse_value(&self, input: &str) -> Result<FlagValue> {
        match &self.value_type {
            FlagType::String => Ok(FlagValue::String(input.to_string())),
            FlagType::Bool => match input.to_lowercase().as_str() {
                "true" | "t" | "1" | "yes" | "y" => Ok(FlagValue::Bool(true)),
                "false" | "f" | "0" | "no" | "n" => Ok(FlagValue::Bool(false)),
                _ => Err(Error::flag_parsing_with_suggestions(
                    format!("Invalid boolean value: '{input}'"),
                    self.name.clone(),
                    vec![
                        "true, false".to_string(),
                        "yes, no".to_string(),
                        "1, 0".to_string(),
                    ],
                )),
            },
            FlagType::Int => input.parse::<i64>().map(FlagValue::Int).map_err(|_| {
                Error::flag_parsing_with_suggestions(
                    format!("Invalid integer value: '{input}'"),
                    self.name.clone(),
                    vec!["a whole number (e.g., 42, -10, 0)".to_string()],
                )
            }),
            FlagType::Float => input.parse::<f64>().map(FlagValue::Float).map_err(|_| {
                Error::flag_parsing_with_suggestions(
                    format!("Invalid float value: '{input}'"),
                    self.name.clone(),
                    vec!["a decimal number (e.g., 3.14, -0.5, 1e10)".to_string()],
                )
            }),
            FlagType::StringSlice | FlagType::StringArray => {
                Ok(FlagValue::StringSlice(vec![input.to_string()]))
            }
            FlagType::Choice(choices) => {
                if choices.contains(&input.to_string()) {
                    Ok(FlagValue::String(input.to_string()))
                } else {
                    Err(Error::flag_parsing_with_suggestions(
                        format!("Invalid choice: '{input}'"),
                        self.name.clone(),
                        choices.clone(),
                    ))
                }
            }
            FlagType::Range(min, max) => {
                let value = input.parse::<i64>().map_err(|_| {
                    Error::flag_parsing_with_suggestions(
                        format!("Invalid integer value: '{input}'"),
                        self.name.clone(),
                        vec![format!("a number between {min} and {max}")],
                    )
                })?;
                if value >= *min && value <= *max {
                    Ok(FlagValue::Int(value))
                } else {
                    Err(Error::flag_parsing_with_suggestions(
                        format!("Value {value} is out of range"),
                        self.name.clone(),
                        vec![format!("a number between {min} and {max} (inclusive)")],
                    ))
                }
            }
            FlagType::File => {
                use std::path::Path;
                let path = Path::new(input);
                if path.exists() && path.is_file() {
                    Ok(FlagValue::String(input.to_string()))
                } else if !path.exists() {
                    Err(Error::flag_parsing_with_suggestions(
                        format!("File not found: '{input}'"),
                        self.name.clone(),
                        vec!["path to an existing file".to_string()],
                    ))
                } else {
                    Err(Error::flag_parsing_with_suggestions(
                        format!("Path exists but is not a file: '{input}'"),
                        self.name.clone(),
                        vec!["path to a regular file (not a directory)".to_string()],
                    ))
                }
            }
            FlagType::Directory => {
                use std::path::Path;
                let path = Path::new(input);
                if path.exists() && path.is_dir() {
                    Ok(FlagValue::String(input.to_string()))
                } else if !path.exists() {
                    Err(Error::flag_parsing_with_suggestions(
                        format!("Directory not found: '{input}'"),
                        self.name.clone(),
                        vec!["path to an existing directory".to_string()],
                    ))
                } else {
                    Err(Error::flag_parsing_with_suggestions(
                        format!("Path exists but is not a directory: '{input}'"),
                        self.name.clone(),
                        vec!["path to a directory (not a file)".to_string()],
                    ))
                }
            }
        }
    }

    /// Validates this flag's constraints against the provided flags
    ///
    /// # Arguments
    ///
    /// * `flag_name` - The name of this flag
    /// * `provided_flags` - Set of flag names that were provided
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if all constraints are satisfied
    ///
    /// # Errors
    ///
    /// Returns `Error::FlagParsing` if any constraint is violated
    pub fn validate_constraints(
        &self,
        flag_name: &str,
        provided_flags: &HashSet<String>,
    ) -> Result<()> {
        for constraint in &self.constraints {
            match constraint {
                FlagConstraint::RequiredIf(other_flag) => {
                    if provided_flags.contains(other_flag) && !provided_flags.contains(flag_name) {
                        return Err(Error::flag_parsing_with_suggestions(
                            format!(
                                "Flag '--{flag_name}' is required when '--{other_flag}' is set"
                            ),
                            flag_name.to_string(),
                            vec![format!("add --{flag_name} <value>")],
                        ));
                    }
                }
                FlagConstraint::ConflictsWith(conflicting_flags) => {
                    if provided_flags.contains(flag_name) {
                        for conflict in conflicting_flags {
                            if provided_flags.contains(conflict) {
                                return Err(Error::flag_parsing_with_suggestions(
                                    format!("Flag '--{flag_name}' conflicts with '--{conflict}'"),
                                    flag_name.to_string(),
                                    vec![format!(
                                        "use either --{flag_name} or --{conflict}, not both"
                                    )],
                                ));
                            }
                        }
                    }
                }
                FlagConstraint::Requires(required_flags) => {
                    if provided_flags.contains(flag_name) {
                        for required in required_flags {
                            if !provided_flags.contains(required) {
                                return Err(Error::flag_parsing_with_suggestions(
                                    format!(
                                        "Flag '--{flag_name}' requires '--{required}' to be set"
                                    ),
                                    flag_name.to_string(),
                                    vec![format!("add --{required} <value>")],
                                ));
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[allow(clippy::approx_constant)]
    const PI: f64 = 3.14;

    #[test]
    fn test_flag_value_conversions() {
        let string_val = FlagValue::String("hello".to_string());
        assert_eq!(string_val.as_string().unwrap(), "hello");
        assert!(string_val.as_bool().is_err());

        let bool_val = FlagValue::Bool(true);
        assert!(bool_val.as_bool().unwrap());
        assert!(bool_val.as_string().is_err());

        let int_val = FlagValue::Int(42);
        assert_eq!(int_val.as_int().unwrap(), 42);
        assert!(int_val.as_float().is_err());

        let float_val = FlagValue::Float(PI);
        assert!((float_val.as_float().unwrap() - PI).abs() < f64::EPSILON);
        assert!(float_val.as_int().is_err());

        let slice_val = FlagValue::StringSlice(vec!["a".to_string(), "b".to_string()]);
        assert_eq!(
            slice_val.as_string_slice().unwrap(),
            &vec!["a".to_string(), "b".to_string()]
        );
        assert!(slice_val.as_string().is_err());
    }

    #[test]
    fn test_flag_parsing() {
        let string_flag = Flag::new("name").value_type(FlagType::String);
        assert_eq!(
            string_flag.parse_value("test").unwrap(),
            FlagValue::String("test".to_string())
        );

        let bool_flag = Flag::new("verbose").value_type(FlagType::Bool);
        assert_eq!(
            bool_flag.parse_value("true").unwrap(),
            FlagValue::Bool(true)
        );
        assert_eq!(
            bool_flag.parse_value("false").unwrap(),
            FlagValue::Bool(false)
        );
        assert_eq!(bool_flag.parse_value("1").unwrap(), FlagValue::Bool(true));
        assert_eq!(bool_flag.parse_value("0").unwrap(), FlagValue::Bool(false));
        assert_eq!(bool_flag.parse_value("yes").unwrap(), FlagValue::Bool(true));
        assert_eq!(bool_flag.parse_value("no").unwrap(), FlagValue::Bool(false));
        assert!(bool_flag.parse_value("invalid").is_err());

        let int_flag = Flag::new("count").value_type(FlagType::Int);
        assert_eq!(int_flag.parse_value("42").unwrap(), FlagValue::Int(42));
        assert_eq!(int_flag.parse_value("-10").unwrap(), FlagValue::Int(-10));
        assert!(int_flag.parse_value("not_a_number").is_err());

        let float_flag = Flag::new("ratio").value_type(FlagType::Float);
        assert_eq!(
            float_flag.parse_value("3.14").unwrap(),
            FlagValue::Float(PI)
        );
        assert_eq!(
            float_flag.parse_value("-2.5").unwrap(),
            FlagValue::Float(-2.5)
        );
        assert!(float_flag.parse_value("not_a_float").is_err());
    }

    #[test]
    fn test_flag_builder() {
        let flag = Flag::new("verbose")
            .short('v')
            .usage("Enable verbose output")
            .default(FlagValue::Bool(false))
            .value_type(FlagType::Bool);

        assert_eq!(flag.name, "verbose");
        assert_eq!(flag.short, Some('v'));
        assert_eq!(flag.usage, "Enable verbose output");
        assert_eq!(flag.default, Some(FlagValue::Bool(false)));
        assert!(!flag.required);
    }

    #[test]
    fn test_choice_flag() {
        let choice_flag = Flag::new("environment").value_type(FlagType::Choice(vec![
            "dev".to_string(),
            "staging".to_string(),
            "prod".to_string(),
        ]));

        assert_eq!(
            choice_flag.parse_value("dev").unwrap(),
            FlagValue::String("dev".to_string())
        );
        assert_eq!(
            choice_flag.parse_value("staging").unwrap(),
            FlagValue::String("staging".to_string())
        );
        assert!(choice_flag.parse_value("test").is_err());
    }

    #[test]
    fn test_range_flag() {
        let range_flag = Flag::new("port").value_type(FlagType::Range(1024, 65535));

        assert_eq!(
            range_flag.parse_value("8080").unwrap(),
            FlagValue::Int(8080)
        );
        assert_eq!(
            range_flag.parse_value("1024").unwrap(),
            FlagValue::Int(1024)
        );
        assert_eq!(
            range_flag.parse_value("65535").unwrap(),
            FlagValue::Int(65535)
        );
        assert!(range_flag.parse_value("80").is_err());
        assert!(range_flag.parse_value("70000").is_err());
        assert!(range_flag.parse_value("not_a_number").is_err());
    }

    #[test]
    fn test_file_flag() {
        use std::fs::File;
        use std::io::Write;
        let temp_file = "test_file_flag.tmp";
        let mut file = File::create(temp_file).unwrap();
        writeln!(file, "test").unwrap();

        let file_flag = Flag::new("config").value_type(FlagType::File);
        assert_eq!(
            file_flag.parse_value(temp_file).unwrap(),
            FlagValue::String(temp_file.to_string())
        );
        assert!(file_flag.parse_value("nonexistent.file").is_err());

        std::fs::remove_file(temp_file).unwrap();
    }

    #[test]
    fn test_directory_flag() {
        let dir_flag = Flag::new("output").value_type(FlagType::Directory);

        // Test with current directory
        assert_eq!(
            dir_flag.parse_value(".").unwrap(),
            FlagValue::String(".".to_string())
        );

        // Test with src directory (should exist in the project)
        assert_eq!(
            dir_flag.parse_value("src").unwrap(),
            FlagValue::String("src".to_string())
        );

        assert!(dir_flag.parse_value("nonexistent_directory").is_err());
    }

    #[test]
    fn test_string_array_flag() {
        let array_flag = Flag::new("tags").value_type(FlagType::StringArray);

        assert_eq!(
            array_flag.parse_value("tag1").unwrap(),
            FlagValue::StringSlice(vec!["tag1".to_string()])
        );
    }

    #[test]
    fn test_flag_constraints() {
        let mut provided_flags = HashSet::new();

        // Test RequiredIf constraint
        let ssl_flag = Flag::new("ssl").constraint(FlagConstraint::RequiredIf("port".to_string()));

        // Should pass when port flag is not set
        assert!(ssl_flag
            .validate_constraints("ssl", &provided_flags)
            .is_ok());

        // Should fail when port is set but ssl is not
        provided_flags.insert("port".to_string());
        assert!(ssl_flag
            .validate_constraints("ssl", &provided_flags)
            .is_err());

        // Should pass when both are set
        provided_flags.insert("ssl".to_string());
        assert!(ssl_flag
            .validate_constraints("ssl", &provided_flags)
            .is_ok());

        // Test ConflictsWith constraint
        let encrypt_flag = Flag::new("encrypt").constraint(FlagConstraint::ConflictsWith(vec![
            "no-encrypt".to_string(),
        ]));

        provided_flags.clear();
        provided_flags.insert("encrypt".to_string());
        assert!(encrypt_flag
            .validate_constraints("encrypt", &provided_flags)
            .is_ok());

        provided_flags.insert("no-encrypt".to_string());
        assert!(encrypt_flag
            .validate_constraints("encrypt", &provided_flags)
            .is_err());

        // Test Requires constraint
        let output_flag =
            Flag::new("output").constraint(FlagConstraint::Requires(vec!["format".to_string()]));

        provided_flags.clear();
        provided_flags.insert("output".to_string());
        assert!(output_flag
            .validate_constraints("output", &provided_flags)
            .is_err());

        provided_flags.insert("format".to_string());
        assert!(output_flag
            .validate_constraints("output", &provided_flags)
            .is_ok());
    }
}
