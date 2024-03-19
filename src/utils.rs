
pub(crate) fn remove_whitespace(s: &str) -> String {
    s.split_whitespace().collect::<String>()
}