
//! Default-extend behavior for arrays in configs
//!
//! The default token `<default>` means:
//! - In the base profile: default values from megaton
//! - In sub-profiles: the base profile values
//!
//! Behavior of the config keys:
//! - The default token splices the default values at the position in the array.
//! - If the key is not specified, then it's treated as `["<default>"]`, meaning:
//!   - In the base profile, it will use the default values from megaton
//!   - In sub-profiles, it will use the same values as default
//! - `[]` means "nothing"; the defaults are excluded
//! - multiple `"<default>"`s are NOT deduplicated
//!
//! The config values are resolved in 2 phases:
//! - First, during profile selection/extension, the values is expanded
//!   to an array that contains the default token, or `None`
//! - Second, the default tokens are expanded to the default values
//!   when the resolved values are needed.

/// The token in the config file to indicate extending default values
pub static DEFAULT_TOKEN: &str = "<default>";

pub trait DefaultToken {
    fn is_default_token(&self) -> bool;
    fn get_default_token() -> Self;
}

#[rustfmt::skip]
const _: () = {
    impl DefaultToken for String {
        fn is_default_token(&self) -> bool { self == DEFAULT_TOKEN }
        fn get_default_token() -> Self { DEFAULT_TOKEN.to_string() }
    }
    impl DefaultToken for &str {
        fn is_default_token(&self) -> bool { *self == DEFAULT_TOKEN }
        fn get_default_token() -> Self { DEFAULT_TOKEN }
    }
};

/// Extend by the default token behavior
pub fn extend_by_default_token<T: DefaultToken + Clone>(current: &mut Option<Vec<T>>, incoming_extend: Option<&Vec<T>>) {
    let Some(incoming_extend) = incoming_extend else {
        return;
    };
    match current {
        None => {
            *current = Some(incoming_extend.clone());
        }
        Some(current) => {
            let mut next = Vec::with_capacity(current.len() + incoming_extend.len());
            for value in incoming_extend {
                if value.is_default_token() {
                    // must clone because <default> is not deduplicated, so there might be more
                    next.extend(current.clone());
                } else {
                    next.push(value.clone());
                }
            }
            *current = next;
        }
    }
}

/// Resolve the default tokens in `values` into `output`
pub fn resolve_default_token<T: DefaultToken + Clone, F: FnOnce() -> Vec<T>>(
    mut output: Vec<T>,
    values: &[T], resolve_fn: F
) -> Vec<T> {
    enum ResolveState<T, F: FnOnce() -> Vec<T>> {
        Pending(F),
        Resolved(Vec<T>),
    }
    let mut default_values = ResolveState::Pending(resolve_fn);
    for value in values {
        if value.is_default_token() {
            match &default_values {
                ResolveState::Pending(_) => {
                    let resolved = match default_values {
                        ResolveState::Pending(f) => {f()},
                        _ => unreachable!()
                    };
                    output.extend_from_slice(&resolved);
                    default_values = ResolveState::Resolved(resolved);
                }
                ResolveState::Resolved(resolved) => {
                    output.extend_from_slice(&resolved);
                }
            }
        } else {
            output.push(value.clone());
        }
    }

    output
}
