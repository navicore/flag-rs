//! Timeout handling for completion functions
//!
//! This module provides utilities to wrap completion functions with timeouts
//! to prevent slow operations from hanging the shell completion experience.

use crate::completion::CompletionResult;
use crate::context::Context;
use crate::error::{Error, Result};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Default timeout for completion operations (2 seconds)
pub const DEFAULT_COMPLETION_TIMEOUT: Duration = Duration::from_secs(2);

/// Wraps a completion function with a timeout
///
/// This function ensures that completion operations don't hang indefinitely
/// by imposing a timeout. If the operation doesn't complete within the timeout,
/// it returns a partial result with a help message indicating the timeout.
///
/// # Arguments
///
/// * `f` - The completion function to wrap
/// * `timeout` - Maximum duration to wait for completion
/// * `ctx` - The context for the completion
/// * `prefix` - The prefix being completed
///
/// # Returns
///
/// Returns the completion result if it completes within the timeout,
/// or a partial result with timeout information if it exceeds the timeout.
pub fn with_timeout<F>(
    f: F,
    timeout: Duration,
    ctx: &Context,
    prefix: &str,
) -> Result<CompletionResult>
where
    F: FnOnce(&Context, &str) -> Result<CompletionResult> + Send + 'static,
{
    // Create shared state for the result
    let result: Arc<Mutex<Option<Result<CompletionResult>>>> = Arc::new(Mutex::new(None));
    let result_clone = Arc::clone(&result);

    // Clone necessary data for the thread
    let ctx_clone = Context::new(ctx.args().to_vec());
    let prefix_clone = prefix.to_string();

    // Spawn the completion function in a separate thread
    let handle = thread::spawn(move || {
        let completion_result = f(&ctx_clone, &prefix_clone);
        if let Ok(mut result_lock) = result_clone.lock() {
            *result_lock = Some(completion_result);
        }
    });

    // Wait for the thread with timeout
    if matches!(handle.join_timeout(timeout), Ok(())) {
        // Thread completed within timeout
        result.lock().map_or_else(
            |_| {
                Err(Error::Completion(
                    "Failed to access completion result".to_string(),
                ))
            },
            |mut result_lock| {
                result_lock.take().unwrap_or_else(|| {
                    Err(Error::Completion(
                        "Completion function did not return a result".to_string(),
                    ))
                })
            },
        )
    } else {
        // Timeout occurred
        let mut partial_result = CompletionResult::new();
        partial_result = partial_result.add_help_text(
            "⚠️  Completion timed out - results may be incomplete. Try a more specific prefix.",
        );

        // If we have any partial results from before the timeout, include them
        if let Ok(result_lock) = result.lock() {
            if let Some(Ok(ref partial)) = *result_lock {
                partial_result = partial_result.merge(partial.clone());
            }
        }

        Ok(partial_result)
    }
}

/// Creates a timeout-wrapped completion function
///
/// This is a convenience function that creates a new completion function
/// with built-in timeout handling.
///
/// # Arguments
///
/// * `timeout` - Maximum duration to wait for completion
/// * `f` - The original completion function
///
/// # Returns
///
/// A new completion function that enforces the timeout
pub fn make_timeout_completion<F>(
    timeout: Duration,
    f: F,
) -> impl Fn(&Context, &str) -> Result<CompletionResult>
where
    F: Fn(&Context, &str) -> Result<CompletionResult> + Clone + Send + 'static,
{
    move |ctx: &Context, prefix: &str| {
        let f_clone = f.clone();
        with_timeout(move |c, p| f_clone(c, p), timeout, ctx, prefix)
    }
}

// Extension trait to add timeout support to threads
trait JoinHandleExt<T>: Sized {
    fn join_timeout(self, timeout: Duration) -> std::result::Result<T, Self>;
}

impl<T> JoinHandleExt<T> for thread::JoinHandle<T> {
    fn join_timeout(self, timeout: Duration) -> std::result::Result<T, Self> {
        let start = std::time::Instant::now();

        loop {
            if self.is_finished() {
                return self.join().map_or_else(|_| panic!("Thread panicked"), Ok);
            }

            if start.elapsed() >= timeout {
                return Err(self);
            }

            thread::sleep(Duration::from_millis(10));
        }
    }
}

impl CompletionResult {
    /// Merges two completion results, combining their values and help messages
    #[must_use]
    pub fn merge(mut self, other: Self) -> Self {
        self.values.extend(other.values);
        self.descriptions.extend(other.descriptions);
        self.active_help.extend(other.active_help);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_completion_with_timeout_success() {
        let ctx = Context::new(vec![]);

        let result = with_timeout(
            |_ctx, prefix| {
                // Fast completion
                Ok(CompletionResult::new()
                    .add("item1")
                    .add("item2")
                    .add(format!("prefix_{prefix}")))
            },
            Duration::from_secs(1),
            &ctx,
            "test",
        );

        assert!(result.is_ok());
        let completion = result.unwrap();
        assert_eq!(completion.values.len(), 3);
        assert!(completion.values.contains(&"prefix_test".to_string()));
    }

    #[test]
    fn test_completion_with_timeout_exceeded() {
        let ctx = Context::new(vec![]);

        let result = with_timeout(
            |_ctx, _prefix| {
                // Slow completion that will timeout
                thread::sleep(Duration::from_secs(2));
                Ok(CompletionResult::new().add("never_returned"))
            },
            Duration::from_millis(100),
            &ctx,
            "test",
        );

        assert!(result.is_ok());
        let completion = result.unwrap();
        // Should have timeout warning in active help
        assert!(!completion.active_help.is_empty());
        assert!(completion.active_help[0]
            .message
            .contains("Completion timed out"));
    }

    #[test]
    fn test_make_timeout_completion() {
        let wrapped = make_timeout_completion(Duration::from_secs(1), |_ctx, prefix| {
            Ok(CompletionResult::new().add(format!("result_{prefix}")))
        });

        let ctx = Context::new(vec![]);
        let result = wrapped(&ctx, "test").unwrap();
        assert_eq!(result.values.len(), 1);
        assert_eq!(result.values[0], "result_test");
    }
}
