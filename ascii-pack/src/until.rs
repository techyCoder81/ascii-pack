/// Helper function for stopping when the first digit of
/// the next chunk is an ascii numeric digit.
pub fn ascii_digit(slice: &str) -> bool {
    slice
        .chars()
        .next()
        .map(|c| c.is_ascii_digit())
        .unwrap_or(true)
}

/// Helper function for stopping when the first digit of
/// the next chunk is an ascii alphanumeric character
pub fn ascii_alphanumeric(slice: &str) -> bool {
    slice
        .chars()
        .next()
        .map(|c| c.is_ascii_alphanumeric())
        .unwrap_or(true)
}

/// Helper function for stopping when the first digit of
/// the next chunk is an ascii alphabetic character.
pub fn ascii_alpha(slice: &str) -> bool {
    slice
        .chars()
        .next()
        .map(|c| c.is_alphabetic())
        .unwrap_or(true)
}

/// Helper function for stopping when the next chunk starts
/// with a specific delimeter
pub fn starts_with(delimeter: &'static str) -> impl Fn(&str) -> bool {
    return move |s| s.starts_with(delimeter);
}

/// Helper function for stopping when there are no more
/// characters left to consume.
pub fn empty(slice: &str) -> bool {
    slice.len() == 0
}
