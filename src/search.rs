use std::collections::BTreeSet;
use walkdir::WalkDir;

#[derive(Debug, Clone, Copy)]
pub enum FuzzySearchAlgorithm {
    LEVENSHTEIN,
    DamerauLevenshtein,
    BITAP,
    JaroWinkler,
}

/// Walks over the directory and returns a vector of tuples containing the file name and the full path.
/// Skip the files with the extensions provided in the exclude_extensions flag.
///
/// # Arguments
///
/// * `exclude_extension_set` - A set of file extensions to exclude from the results.
/// * `focus_extension_set` - A set of file extensions to include in the results. If empty, all extensions except those in `exclude_extension_set` are included.
///
/// # Returns
///
/// A vector of tuples where each tuple contains the file name (without extension) and the full path.
pub fn walk_directory(
    exclude_extension_set: BTreeSet<String>,
    focus_extension_set: BTreeSet<String>,
) -> Vec<(String, String)> {
    let mut files = Vec::new();
    for entry in WalkDir::new(".")
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| !e.file_type().is_dir())
    {
        let file_name: String = String::from(entry.file_name().to_string_lossy());
        let full_path: String = String::from(entry.path().to_string_lossy());
        let chunks: Vec<&str> = file_name.split('.').collect();
        let raw_file_name: String = chunks[..chunks.len() - 1].join(".");
        if focus_extension_set.is_empty() {
            if chunks
                .last()
                .map_or(true, |ext| !exclude_extension_set.contains(*ext))
            {
                files.push((raw_file_name, full_path));
            }
        } else {
            if chunks
                .last()
                .map_or(false, |ext| focus_extension_set.contains(*ext))
            {
                files.push((raw_file_name, full_path));
            }
        }
    }

    return files;
}

/// Scores the similarity between a query and a file name using the specified fuzzy search algorithm.
///
/// # Arguments
///
/// * `query` - The search query string.
/// * `file_name` - The file name to compare against the query.
/// * `scorer` - The fuzzy search algorithm to use for scoring.
///
/// # Returns
///
/// A result containing the similarity score as `u32` if the algorithm is implemented, otherwise an error message.
pub fn score_fuzzy_search(
    query: String,
    file_name: String,
    scorer: FuzzySearchAlgorithm,
) -> Result<u32, String> {
    match scorer {
        FuzzySearchAlgorithm::DamerauLevenshtein => {
            Ok(damerau_levenshtein_distance(query, file_name))
        }
        _ => Err(format!("{:?} Algorithm not implemented", scorer)),
    }
}

/// Computes the Damerau-Levenshtein distance between two strings.
///
/// # Arguments
///
/// * `query` - The first string.
/// * `file_name` - The second string.
///
/// # Returns
///
/// The Damerau-Levenshtein distance as `u32`.
fn damerau_levenshtein_distance(query: String, file_name: String) -> u32 {
    let n: usize = query.len();
    let m: usize = file_name.len();

    let mut dp: Vec<Vec<u32>> = vec![vec![0; m + 1]; n + 1];
    for i in 0..=n {
        dp[i][0] = i as u32;
    }
    for j in 0..=m {
        dp[0][j] = j as u32;
    }
    for i in 1..=n {
        for j in 1..=m {
            if query.chars().nth(i - 1) == file_name.chars().nth(j - 1) {
                dp[i][j] = dp[i - 1][j - 1];
            } else {
                dp[i][j] =
                    1 + std::cmp::min(dp[i - 1][j], std::cmp::min(dp[i][j - 1], dp[i - 1][j - 1]));
            }
            if i > 1
                && j > 1
                && query.chars().nth(i - 1) == file_name.chars().nth(j - 2)
                && query.chars().nth(i - 2) == file_name.chars().nth(j - 1)
            {
                dp[i][j] = std::cmp::min(dp[i][j], dp[i - 2][j - 2] + 1);
            }
        }
    }
    return dp[n][m];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_damerau_levenshtein_distance() {
        assert_eq!(
            damerau_levenshtein_distance("irks".to_string(), "risk".to_string()),
            2
        );
        assert_eq!(
            damerau_levenshtein_distance("geeks".to_string(), "forgeeks".to_string()),
            3
        );
    }
}
