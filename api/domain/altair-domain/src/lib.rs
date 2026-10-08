//! Placeholder domain crate. Replace with real domain rules.

/// Name of the product this workspace serves.
pub fn product_name() -> &'static str {
    "Palmy"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_the_product() {
        assert_eq!(product_name(), "Palmy");
    }
}
