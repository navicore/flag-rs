//! Command and flag suggestion support
//!
//! This module provides "did you mean" style suggestions when users
//! make typos in command or flag names.

/// Calculates the Levenshtein distance between two strings
///
/// This is the minimum number of single-character edits (insertions,
/// deletions, or substitutions) required to change one string into another.
pub fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.len();
    let len2 = s2.len();

    // Handle empty strings
    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }

    // Create a 2D vector for dynamic programming
    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    // Initialize first row and column
    for (i, row) in matrix.iter_mut().enumerate().take(len1 + 1) {
        row[0] = i;
    }
    for j in 0..=len2 {
        matrix[0][j] = j;
    }

    // Fill the matrix
    let chars1: Vec<char> = s1.chars().collect();
    let chars2: Vec<char> = s2.chars().collect();

    for i in 1..=len1 {
        for j in 1..=len2 {
            let cost = usize::from(chars1[i - 1] != chars2[j - 1]);

            matrix[i][j] = std::cmp::min(
                std::cmp::min(
                    matrix[i - 1][j] + 1, // deletion
                    matrix[i][j - 1] + 1, // insertion
                ),
                matrix[i - 1][j - 1] + cost, // substitution
            );
        }
    }

    matrix[len1][len2]
}

/// Finds suggestions from a list of candidates based on similarity to input
///
/// Returns candidates that have a Levenshtein distance less than or equal
/// to the threshold, sorted by distance (closest first).
pub fn find_suggestions(input: &str, candidates: &[String], max_distance: usize) -> Vec<String> {
    let mut suggestions: Vec<(String, usize)> = candidates
        .iter()
        .map(|candidate| {
            let distance = levenshtein_distance(input, candidate);
            (candidate.clone(), distance)
        })
        .filter(|(_, distance)| *distance <= max_distance)
        .collect();

    // Sort by distance, then alphabetically
    suggestions.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)));

    suggestions.into_iter().map(|(name, _)| name).collect()
}

/// Default maximum Levenshtein distance for suggestions
pub const DEFAULT_SUGGESTION_DISTANCE: usize = 2;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_distance() {
        // Identical strings
        assert_eq!(levenshtein_distance("hello", "hello"), 0);

        // One character difference
        assert_eq!(levenshtein_distance("hello", "hallo"), 1);
        assert_eq!(levenshtein_distance("hello", "ello"), 1);
        assert_eq!(levenshtein_distance("hello", "helloo"), 1);

        // Multiple differences
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
        assert_eq!(levenshtein_distance("saturday", "sunday"), 3);

        // Empty strings
        assert_eq!(levenshtein_distance("", "hello"), 5);
        assert_eq!(levenshtein_distance("hello", ""), 5);
        assert_eq!(levenshtein_distance("", ""), 0);
    }

    #[test]
    fn test_find_suggestions() {
        let candidates = vec![
            "start".to_string(),
            "status".to_string(),
            "stop".to_string(),
            "restart".to_string(),
            "help".to_string(),
        ];

        // Close match - distance 2 includes "start" (1), "status" (2), and "stop" (2)
        let suggestions = find_suggestions("stat", &candidates, 2);
        assert_eq!(suggestions, vec!["start", "status", "stop"]);

        // Exact match
        let suggestions = find_suggestions("stop", &candidates, 2);
        assert_eq!(suggestions, vec!["stop"]);

        // No matches within distance
        let suggestions = find_suggestions("xyz", &candidates, 2);
        assert!(suggestions.is_empty());

        // Multiple matches, sorted by distance
        let suggestions = find_suggestions("sart", &candidates, 2);
        assert_eq!(suggestions, vec!["start"]); // distance 1
    }

    #[test]
    fn test_case_sensitivity() {
        // Currently case-sensitive
        assert_eq!(levenshtein_distance("Hello", "hello"), 1);

        let candidates = vec!["Start".to_string(), "start".to_string()];
        let suggestions = find_suggestions("start", &candidates, 0);
        assert_eq!(suggestions, vec!["start"]);
    }
}
