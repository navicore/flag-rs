use crate::error::{Error, Result};

#[derive(Clone, Debug, PartialEq)]
pub enum FlagValue {
    String(String),
    Bool(bool),
    Int(i64),
    Float(f64),
    StringSlice(Vec<String>),
}

impl FlagValue {
    pub fn as_string(&self) -> Result<&String> {
        match self {
            Self::String(s) => Ok(s),
            _ => Err(Error::FlagParsing("Flag is not a string".to_string())),
        }
    }

    pub fn as_bool(&self) -> Result<bool> {
        match self {
            Self::Bool(b) => Ok(*b),
            _ => Err(Error::FlagParsing("Flag is not a bool".to_string())),
        }
    }

    pub fn as_int(&self) -> Result<i64> {
        match self {
            Self::Int(i) => Ok(*i),
            _ => Err(Error::FlagParsing("Flag is not an integer".to_string())),
        }
    }

    pub fn as_float(&self) -> Result<f64> {
        match self {
            Self::Float(f) => Ok(*f),
            _ => Err(Error::FlagParsing("Flag is not a float".to_string())),
        }
    }

    pub fn as_string_slice(&self) -> Result<&Vec<String>> {
        match self {
            Self::StringSlice(v) => Ok(v),
            _ => Err(Error::FlagParsing("Flag is not a string slice".to_string())),
        }
    }
}

#[derive(Clone)]
pub struct Flag {
    pub name: String,
    pub short: Option<char>,
    pub usage: String,
    pub default: Option<FlagValue>,
    pub required: bool,
    pub value_type: FlagType,
}

#[derive(Clone, Debug)]
pub enum FlagType {
    String,
    Bool,
    Int,
    Float,
    StringSlice,
}

impl Flag {
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

    #[must_use]
    pub const fn short(mut self, short: char) -> Self {
        self.short = Some(short);
        self
    }

    #[must_use]
    pub fn usage(mut self, usage: impl Into<String>) -> Self {
        self.usage = usage.into();
        self
    }

    #[must_use]
    pub fn default(mut self, value: FlagValue) -> Self {
        self.default = Some(value);
        self
    }

    #[must_use]
    pub const fn required(mut self) -> Self {
        self.required = true;
        self
    }

    #[must_use]
    pub const fn value_type(mut self, value_type: FlagType) -> Self {
        self.value_type = value_type;
        self
    }

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
        assert_eq!(float_val.as_float().unwrap(), PI);
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
