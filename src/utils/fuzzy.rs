//! Fuzzy string matching utilities.
//!
//! This module provides fuzzy string matching functionality similar to
//! Python's `difflib.get_close_matches`. It uses the Jaro-Winkler similarity
//! metric from the `strsim` crate for efficient fuzzy matching.

use strsim::jaro_winkler;

/// Default number of close matches to return.
pub const DEFAULT_N: usize = 3;

/// Default minimum similarity cutoff (0.0 to 1.0).
pub const DEFAULT_CUTOFF: f64 = 0.6;

/// Get close matches for a word from a list of possibilities.
///
/// This function is similar to Python's `difflib.get_close_matches`.
/// It returns a list of the best "good enough" matches from the possibilities.
///
/// # Arguments
///
/// * `word` - The word to find matches for
/// * `possibilities` - A slice of possible matches
/// * `n` - Maximum number of close matches to return
/// * `cutoff` - Minimum similarity score (0.0 to 1.0) for a match to be included
///
/// # Returns
///
/// A vector of close matches, sorted by similarity (best first).
///
/// # Example
///
/// ```
/// use oops::utils::fuzzy::get_close_matches;
///
/// let words = vec!["apple".to_string(), "apply".to_string(), "banana".to_string()];
/// let matches = get_close_matches("appel", &words, 3, 0.6);
/// assert!(matches.contains(&"apple".to_string()));
/// ```
pub fn get_close_matches(
    word: &str,
    possibilities: &[String],
    n: usize,
    cutoff: f64,
) -> Vec<String> {
    if possibilities.is_empty() || n == 0 {
        return Vec::new();
    }

    // Compute similarity scores for all possibilities
    let mut scored: Vec<(f64, &String)> = possibilities
        .iter()
        .map(|p| (jaro_winkler(word, p), p))
        .filter(|(score, _)| *score >= cutoff)
        .collect();

    // Sort by score descending (highest similarity first)
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    // Take the top n matches
    scored.into_iter().take(n).map(|(_, s)| s.clone()).collect()
}

/// Get close matches with default parameters.
///
/// This is a convenience function that uses default values for `n` and `cutoff`.
///
/// # Arguments
///
/// * `word` - The word to find matches for
/// * `possibilities` - A slice of possible matches
///
/// # Returns
///
/// A vector of up to 3 close matches with similarity >= 0.6.
pub fn get_close_matches_default(word: &str, possibilities: &[String]) -> Vec<String> {
    get_close_matches(word, possibilities, DEFAULT_N, DEFAULT_CUTOFF)
}

/// Get the closest match from a list of possibilities.
///
/// Returns the single best match from the possibilities, with optional
/// fallback to the first element if no match meets the cutoff threshold.
///
/// # Arguments
///
/// * `word` - The word to find a match for
/// * `possibilities` - A slice of possible matches
/// * `cutoff` - Minimum similarity score (0.0 to 1.0) for a match
/// * `fallback_to_first` - If true, return the first possibility when no match is found
///
/// # Returns
///
/// * `Some(String)` - The closest match if found, or the first possibility if fallback is enabled
/// * `None` - If no match is found and fallback is disabled, or if possibilities is empty
///
/// # Example
///
/// ```
/// use oops::utils::fuzzy::get_closest;
///
/// let words = vec!["apple".to_string(), "apply".to_string()];
/// let closest = get_closest("appel", &words, 0.6, true);
/// assert_eq!(closest, Some("apple".to_string()));
/// ```
pub fn get_closest(
    word: &str,
    possibilities: &[String],
    cutoff: f64,
    fallback_to_first: bool,
) -> Option<String> {
    if possibilities.is_empty() {
        return None;
    }

    // Try to get the closest match
    let matches = get_close_matches(word, possibilities, 1, cutoff);

    if let Some(m) = matches.into_iter().next() {
        Some(m)
    } else if fallback_to_first {
        Some(possibilities[0].clone())
    } else {
        None
    }
}

/// Get the closest match with a default cutoff.
///
/// Uses the default cutoff of 0.6.
///
/// # Arguments
///
/// * `word` - The word to find a match for
/// * `possibilities` - A slice of possible matches
/// * `fallback_to_first` - If true, return the first possibility when no match is found
///
/// # Returns
///
/// The closest match, or None if no match is found and fallback is disabled.
pub fn get_closest_default(
    word: &str,
    possibilities: &[String],
    fallback_to_first: bool,
) -> Option<String> {
    get_closest(word, possibilities, DEFAULT_CUTOFF, fallback_to_first)
}

/// Calculate the similarity between two strings.
///
/// Uses the Jaro-Winkler distance metric, which is particularly good for
/// short strings and typo detection.
///
/// # Arguments
///
/// * `a` - First string
/// * `b` - Second string
///
/// # Returns
///
/// A similarity score between 0.0 (completely different) and 1.0 (identical).
pub fn similarity(a: &str, b: &str) -> f64 {
    jaro_winkler(a, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_close_matches_basic() {
        let possibilities = vec![
            "apple".to_string(),
            "apply".to_string(),
            "banana".to_string(),
            "orange".to_string(),
        ];

        let matches = get_close_matches("appel", &possibilities, 3, 0.6);
        assert!(!matches.is_empty());
        // "apple" should be one of the matches since it's very similar to "appel"
        assert!(matches.contains(&"apple".to_string()));
    }

    #[test]
    fn test_get_close_matches_empty_possibilities() {
        let possibilities: Vec<String> = vec![];
        let matches = get_close_matches("test", &possibilities, 3, 0.6);
        assert!(matches.is_empty());
    }

    #[test]
    fn test_get_close_matches_no_matches() {
        let possibilities = vec!["xyz".to_string(), "abc".to_string()];
        // With a very high cutoff, nothing should match
        let matches = get_close_matches("hello", &possibilities, 3, 0.99);
        assert!(matches.is_empty());
    }

    #[test]
    fn test_get_close_matches_exact_match() {
        let possibilities = vec!["hello".to_string(), "world".to_string()];
        let matches = get_close_matches("hello", &possibilities, 3, 0.6);
        assert_eq!(matches.first(), Some(&"hello".to_string()));
    }

    #[test]
    fn test_get_close_matches_respects_n() {
        let possibilities = vec![
            "test1".to_string(),
            "test2".to_string(),
            "test3".to_string(),
            "test4".to_string(),
            "test5".to_string(),
        ];
        let matches = get_close_matches("test", &possibilities, 2, 0.5);
        assert!(matches.len() <= 2);
    }

    #[test]
    fn test_get_closest_with_match() {
        let possibilities = vec!["apple".to_string(), "banana".to_string()];
        let closest = get_closest("appel", &possibilities, 0.6, false);
        assert_eq!(closest, Some("apple".to_string()));
    }

    #[test]
    fn test_get_closest_fallback() {
        let possibilities = vec!["xyz".to_string(), "abc".to_string()];
        // With high cutoff, nothing matches, but fallback should return first
        let closest = get_closest("hello", &possibilities, 0.99, true);
        assert_eq!(closest, Some("xyz".to_string()));
    }

    #[test]
    fn test_get_closest_no_fallback() {
        let possibilities = vec!["xyz".to_string(), "abc".to_string()];
        // With high cutoff and no fallback, should return None
        let closest = get_closest("hello", &possibilities, 0.99, false);
        assert!(closest.is_none());
    }

    #[test]
    fn test_get_closest_empty_possibilities() {
        let possibilities: Vec<String> = vec![];
        let closest = get_closest("test", &possibilities, 0.6, true);
        assert!(closest.is_none());
    }

    #[test]
    fn test_similarity() {
        // Identical strings should have similarity of 1.0
        assert!((similarity("hello", "hello") - 1.0).abs() < f64::EPSILON);

        // Similar strings should have high similarity
        let sim = similarity("hello", "hallo");
        assert!(sim > 0.8);

        // Very different strings should have low similarity
        let sim = similarity("hello", "xyz");
        assert!(sim < 0.5);
    }

    #[test]
    fn test_git_command_matching() {
        // Test case similar to what oops would use
        let git_commands = vec![
            "status".to_string(),
            "commit".to_string(),
            "push".to_string(),
            "pull".to_string(),
            "checkout".to_string(),
            "branch".to_string(),
            "stash".to_string(),
        ];

        // Common typos
        let matches = get_close_matches("statsu", &git_commands, 1, 0.6);
        assert_eq!(matches.first(), Some(&"status".to_string()));

        let matches = get_close_matches("comit", &git_commands, 1, 0.6);
        assert_eq!(matches.first(), Some(&"commit".to_string()));

        let matches = get_close_matches("checkou", &git_commands, 1, 0.6);
        assert_eq!(matches.first(), Some(&"checkout".to_string()));
    }
}
