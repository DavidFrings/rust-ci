//! Fixture crate for the self-test of `rust-ci`. It only has to compile,
//! pass clippy with the lints from `Cargo.toml` and run one doctest.

/// Adds two numbers.
///
/// ```
/// assert_eq!(hello::add(1, 2), 3);
/// ```
#[must_use]
pub fn add(a: u32, b: u32) -> u32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::add;

    #[test]
    fn adds() {
        assert_eq!(add(2, 2), 4);
    }
}
