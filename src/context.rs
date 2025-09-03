//! Context for passing data between commands
//!
//! The context module provides a way to pass data between parent and child
//! commands, including parsed arguments, flags, and arbitrary typed values.

use std::any::{Any, TypeId};
use std::collections::HashMap;

/// Context passed to command handlers
///
/// `Context` provides access to:
/// - Command arguments
/// - Parsed flag values
/// - Arbitrary typed values for sharing state between commands
///
/// # Examples
///
/// ```
/// use flag_rs::context::Context;
/// use std::collections::HashMap;
///
/// // Create a context with arguments
/// let mut ctx = Context::new(vec!["file1.txt".to_string(), "file2.txt".to_string()]);
///
/// // Access arguments
/// assert_eq!(ctx.args(), &["file1.txt", "file2.txt"]);
///
/// // Set and retrieve flags
/// ctx.set_flag("verbose".to_string(), "true".to_string());
/// assert_eq!(ctx.flag("verbose"), Some(&"true".to_string()));
///
/// // Store typed values
/// #[derive(Debug, PartialEq)]
/// struct Config {
///     api_key: String,
/// }
///
/// ctx.set(Config { api_key: "secret".to_string() });
/// let config = ctx.get::<Config>().unwrap();
/// assert_eq!(config.api_key, "secret");
/// ```
pub struct Context {
    args: Vec<String>,
    flags: HashMap<String, String>,
    values: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl Context {
    /// Creates a new context with the given arguments
    ///
    /// # Arguments
    ///
    /// * `args` - The command-line arguments (without the command path)
    pub fn new(args: Vec<String>) -> Self {
        Self {
            args,
            flags: HashMap::new(),
            values: HashMap::new(),
        }
    }

    /// Returns a slice of the command arguments
    ///
    /// # Examples
    ///
    /// ```
    /// use flag_rs::context::Context;
    ///
    /// let ctx = Context::new(vec!["file.txt".to_string()]);
    /// assert_eq!(ctx.args(), &["file.txt"]);
    /// ```
    pub fn args(&self) -> &[String] {
        &self.args
    }

    /// Returns a mutable reference to the command arguments
    ///
    /// # Examples
    ///
    /// ```
    /// use flag_rs::context::Context;
    ///
    /// let mut ctx = Context::new(vec!["file.txt".to_string()]);
    /// ctx.args_mut().push("another.txt".to_string());
    /// assert_eq!(ctx.args().len(), 2);
    /// ```
    pub fn args_mut(&mut self) -> &mut Vec<String> {
        &mut self.args
    }

    /// Gets the value of a flag by name
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the flag
    ///
    /// # Returns
    ///
    /// Returns `Some(&String)` if the flag exists, `None` otherwise
    pub fn flag(&self, name: &str) -> Option<&String> {
        self.flags.get(name)
    }

    /// Sets a flag value
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the flag
    /// * `value` - The value to set
    pub fn set_flag(&mut self, name: String, value: String) {
        self.flags.insert(name, value);
    }

    /// Returns a reference to all flags
    pub fn flags(&self) -> &HashMap<String, String> {
        &self.flags
    }

    /// Gets a flag value as a boolean
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the flag
    ///
    /// # Returns
    ///
    /// Returns `Some(bool)` if the flag exists and can be parsed as a boolean, `None` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use flag_rs::context::Context;
    ///
    /// let mut ctx = Context::new(vec![]);
    /// ctx.set_flag("verbose".to_string(), "true".to_string());
    /// ctx.set_flag("debug".to_string(), "false".to_string());
    ///
    /// assert_eq!(ctx.flag_bool("verbose"), Some(true));
    /// assert_eq!(ctx.flag_bool("debug"), Some(false));
    /// assert_eq!(ctx.flag_bool("missing"), None);
    /// ```
    pub fn flag_bool(&self, name: &str) -> Option<bool> {
        self.flag(name)
            .and_then(|v| match v.to_lowercase().as_str() {
                "true" | "t" | "1" | "yes" | "y" => Some(true),
                "false" | "f" | "0" | "no" | "n" => Some(false),
                _ => None,
            })
    }

    /// Gets a flag value as an integer
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the flag
    ///
    /// # Returns
    ///
    /// Returns `Some(i64)` if the flag exists and can be parsed as an integer, `None` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use flag_rs::context::Context;
    ///
    /// let mut ctx = Context::new(vec![]);
    /// ctx.set_flag("port".to_string(), "8080".to_string());
    ///
    /// assert_eq!(ctx.flag_int("port"), Some(8080));
    /// assert_eq!(ctx.flag_int("missing"), None);
    /// ```
    pub fn flag_int(&self, name: &str) -> Option<i64> {
        self.flag(name).and_then(|v| v.parse().ok())
    }

    /// Gets a flag value as a float
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the flag
    ///
    /// # Returns
    ///
    /// Returns `Some(f64)` if the flag exists and can be parsed as a float, `None` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use flag_rs::context::Context;
    ///
    /// let mut ctx = Context::new(vec![]);
    /// ctx.set_flag("ratio".to_string(), "0.75".to_string());
    ///
    /// assert_eq!(ctx.flag_float("ratio"), Some(0.75));
    /// assert_eq!(ctx.flag_float("missing"), None);
    /// ```
    pub fn flag_float(&self, name: &str) -> Option<f64> {
        self.flag(name).and_then(|v| v.parse().ok())
    }

    /// Gets a flag value as a string, returning a default if not present
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the flag
    /// * `default` - The default value to return if the flag is not set
    ///
    /// # Returns
    ///
    /// Returns the flag value if present, or the default value
    ///
    /// # Examples
    ///
    /// ```
    /// use flag_rs::context::Context;
    ///
    /// let mut ctx = Context::new(vec![]);
    /// ctx.set_flag("env".to_string(), "production".to_string());
    ///
    /// assert_eq!(ctx.flag_str_or("env", "development"), "production");
    /// assert_eq!(ctx.flag_str_or("missing", "development"), "development");
    /// ```
    pub fn flag_str_or<'a>(&'a self, name: &str, default: &'a str) -> &'a str {
        self.flag(name).map_or(default, String::as_str)
    }

    /// Gets a flag value as a boolean, returning a default if not present
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the flag
    /// * `default` - The default value to return if the flag is not set or cannot be parsed
    ///
    /// # Examples
    ///
    /// ```
    /// use flag_rs::context::Context;
    ///
    /// let mut ctx = Context::new(vec![]);
    /// ctx.set_flag("verbose".to_string(), "true".to_string());
    ///
    /// assert_eq!(ctx.flag_bool_or("verbose", false), true);
    /// assert_eq!(ctx.flag_bool_or("missing", false), false);
    /// ```
    pub fn flag_bool_or(&self, name: &str, default: bool) -> bool {
        self.flag_bool(name).unwrap_or(default)
    }

    /// Gets a flag value as an integer, returning a default if not present
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the flag
    /// * `default` - The default value to return if the flag is not set or cannot be parsed
    ///
    /// # Examples
    ///
    /// ```
    /// use flag_rs::context::Context;
    ///
    /// let mut ctx = Context::new(vec![]);
    /// ctx.set_flag("port".to_string(), "8080".to_string());
    ///
    /// assert_eq!(ctx.flag_int_or("port", 3000), 8080);
    /// assert_eq!(ctx.flag_int_or("missing", 3000), 3000);
    /// ```
    pub fn flag_int_or(&self, name: &str, default: i64) -> i64 {
        self.flag_int(name).unwrap_or(default)
    }

    /// Gets a flag value as a float, returning a default if not present
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the flag
    /// * `default` - The default value to return if the flag is not set or cannot be parsed
    ///
    /// # Examples
    ///
    /// ```
    /// use flag_rs::context::Context;
    ///
    /// let mut ctx = Context::new(vec![]);
    /// ctx.set_flag("ratio".to_string(), "0.75".to_string());
    ///
    /// assert_eq!(ctx.flag_float_or("ratio", 0.5), 0.75);
    /// assert_eq!(ctx.flag_float_or("missing", 0.5), 0.5);
    /// ```
    pub fn flag_float_or(&self, name: &str, default: f64) -> f64 {
        self.flag_float(name).unwrap_or(default)
    }

    /// Stores a typed value in the context
    ///
    /// Values are stored by their type, so only one value of each type
    /// can be stored at a time. Storing a new value of the same type
    /// will overwrite the previous value.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type of value to store (must be `Send + Sync`)
    ///
    /// # Examples
    ///
    /// ```
    /// use flag_rs::context::Context;
    ///
    /// struct ApiClient {
    ///     endpoint: String,
    /// }
    ///
    /// let mut ctx = Context::new(vec![]);
    /// ctx.set(ApiClient { endpoint: "https://api.example.com".to_string() });
    /// ```
    pub fn set<T: Any + Send + Sync>(&mut self, value: T) {
        self.values.insert(TypeId::of::<T>(), Box::new(value));
    }

    /// Retrieves a typed value from the context
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type of value to retrieve
    ///
    /// # Returns
    ///
    /// Returns `Some(&T)` if a value of that type exists, `None` otherwise
    pub fn get<T: Any + Send + Sync>(&self) -> Option<&T> {
        self.values
            .get(&TypeId::of::<T>())
            .and_then(|v| (**v).downcast_ref())
    }

    /// Retrieves a mutable reference to a typed value from the context
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type of value to retrieve
    ///
    /// # Returns
    ///
    /// Returns `Some(&mut T)` if a value of that type exists, `None` otherwise
    pub fn get_mut<T: Any + Send + Sync>(&mut self) -> Option<&mut T> {
        self.values
            .get_mut(&TypeId::of::<T>())
            .and_then(|v| (**v).downcast_mut())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_args() {
        let args = vec!["arg1".to_string(), "arg2".to_string()];
        let mut ctx = Context::new(args.clone());
        assert_eq!(ctx.args(), &args);

        ctx.args_mut().push("arg3".to_string());
        assert_eq!(ctx.args().len(), 3);
    }

    #[test]
    fn test_context_flags() {
        let mut ctx = Context::new(vec![]);

        ctx.set_flag("verbose".to_string(), "true".to_string());
        ctx.set_flag("output".to_string(), "json".to_string());

        assert_eq!(ctx.flag("verbose"), Some(&"true".to_string()));
        assert_eq!(ctx.flag("output"), Some(&"json".to_string()));
        assert_eq!(ctx.flag("nonexistent"), None);
    }

    #[test]
    fn test_context_values() {
        #[derive(Debug, PartialEq)]
        struct Config {
            timeout: u32,
        }

        let mut ctx = Context::new(vec![]);
        let config = Config { timeout: 30 };

        ctx.set(config);

        assert_eq!(ctx.get::<Config>(), Some(&Config { timeout: 30 }));
        assert_eq!(ctx.get::<String>(), None);

        if let Some(cfg) = ctx.get_mut::<Config>() {
            cfg.timeout = 60;
        }

        assert_eq!(ctx.get::<Config>(), Some(&Config { timeout: 60 }));
    }
}
