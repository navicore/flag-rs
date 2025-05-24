use std::any::{Any, TypeId};
use std::collections::HashMap;

pub struct Context {
    args: Vec<String>,
    flags: HashMap<String, String>,
    values: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl Context {
    pub fn new(args: Vec<String>) -> Self {
        Self {
            args,
            flags: HashMap::new(),
            values: HashMap::new(),
        }
    }

    pub fn args(&self) -> &[String] {
        &self.args
    }

    pub fn args_mut(&mut self) -> &mut Vec<String> {
        &mut self.args
    }

    pub fn flag(&self, name: &str) -> Option<&String> {
        self.flags.get(name)
    }

    pub fn set_flag(&mut self, name: String, value: String) {
        self.flags.insert(name, value);
    }

    pub fn flags(&self) -> &HashMap<String, String> {
        &self.flags
    }

    pub fn set<T: Any + Send + Sync>(&mut self, value: T) {
        self.values.insert(TypeId::of::<T>(), Box::new(value));
    }

    pub fn get<T: Any + Send + Sync>(&self) -> Option<&T> {
        self.values
            .get(&TypeId::of::<T>())
            .and_then(|v| v.downcast_ref())
    }

    pub fn get_mut<T: Any + Send + Sync>(&mut self) -> Option<&mut T> {
        self.values
            .get_mut(&TypeId::of::<T>())
            .and_then(|v| v.downcast_mut())
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
