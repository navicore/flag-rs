//! Flag system for command-line argument parsing
//!
//! This module provides a flexible flag parsing system that supports:
//! - Multiple value types (string, bool, int, float, string slice)
//! - Short and long flag names
//! - Required and optional flags
//! - Default values
//! - Hierarchical flag inheritance from parent commands

use crate::error::{Error, Result};

/// Represents the value of a parsed flag
///
/// `FlagValue` is an enum that can hold different types of values
/// that flags can have. This allows for type-safe access to flag values.
///
/// # Examples
///
/// ```
/// use flag::flag::{FlagValue, FlagType, Flag};
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
    /// use flag::flag::FlagValue;
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
            _ => Err(Error::FlagParsing("Flag is not a string".to_string())),
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
            _ => Err(Error::FlagParsing("Flag is not a bool".to_string())),
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
            _ => Err(Error::FlagParsing("Flag is not an integer".to_string())),
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
            _ => Err(Error::FlagParsing("Flag is not a float".to_string())),
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
            _ => Err(Error::FlagParsing("Flag is not a string slice".to_string())),
        }
    }
}

/// Represents a command-line flag
///
/// A `Flag` defines a command-line option that can be passed to a command.
/// Flags can have both long names (e.g., `--verbose`) and short names (e.g., `-v`).
///
/// # Examples
///
/// ```
/// use flag::flag::{Flag, FlagType, FlagValue};
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
#[derive(Clone)]
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
}

/// Represents the type of value a flag accepts
///
/// This enum determines how flag values are parsed from string input.
#[derive(Clone, Debug)]
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
}

impl Flag {
    /// Creates a new flag with the given name
    ///
    /// # Examples
    ///
    /// ```
    /// use flag::flag::Flag;
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
        }
    }

    /// Sets the short name for this flag
    ///
    /// # Examples
    ///
    /// ```
    /// use flag::flag::Flag;
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
    /// use flag::flag::Flag;
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
    /// use flag::flag::{Flag, FlagValue};
    ///
    /// let flag = Flag::new("count").default(FlagValue::Int(10));
    /// assert_eq!(flag.default, Some(FlagValue::Int(10)));
    /// ```
    #[must_use]
    pub fn default(mut self, value: FlagValue) -> Self {
        self.default = Some(value);
        self
    }

    /// Marks this flag as required
    ///
    /// # Examples
    ///
    /// ```
    /// use flag::flag::Flag;
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
    /// use flag::flag::{Flag, FlagType};
    ///
    /// let flag = Flag::new("count").value_type(FlagType::Int);
    /// ```
    #[must_use]
    pub const fn value_type(mut self, value_type: FlagType) -> Self {
        self.value_type = value_type;
        self
    }

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
    /// use flag::flag::{Flag, FlagType, FlagValue};
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
        match self.value_type {
            FlagType::String => Ok(FlagValue::String(input.to_string())),
            FlagType::Bool => match input.to_lowercase().as_str() {
                "true" | "t" | "1" | "yes" | "y" => Ok(FlagValue::Bool(true)),
                "false" | "f" | "0" | "no" | "n" => Ok(FlagValue::Bool(false)),
                _ => Err(Error::FlagParsing(format!(
                    "Invalid boolean value: {input}"
                ))),
            },
            FlagType::Int => input
                .parse::<i64>()
                .map(FlagValue::Int)
                .map_err(|_| Error::FlagParsing(format!("Invalid integer value: {input}"))),
            FlagType::Float => input
                .parse::<f64>()
                .map(FlagValue::Float)
                .map_err(|_| Error::FlagParsing(format!("Invalid float value: {input}"))),
            FlagType::StringSlice => Ok(FlagValue::StringSlice(vec![input.to_string()])),
        }
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
}
