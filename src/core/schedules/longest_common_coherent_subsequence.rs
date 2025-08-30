use std::ops::Deref;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct LongestCommonCoherentSubsequence {
    pub sec1_start: usize,
    pub sec2_start: usize,
    pub len: usize,
}

// TODO: maybe extract into separate crate
pub fn longest_common_coherent_subsequence<T: PartialEq, C1: Deref<Target = [T]>, C2: Deref<Target = [T]>>(seq1: C1, seq2: C2) -> LongestCommonCoherentSubsequence {
    let new_vec = || {
        let mut vec = Vec::<usize>::with_capacity(seq2.len() + 1);
        vec.resize(seq2.len() + 1, 0);
        vec
    };

    let seq1: &[T] = &*seq1;
    let seq2: &[T] = &*seq2;

    let mut result = LongestCommonCoherentSubsequence {
        sec1_start: 0,
        sec2_start: 0,
        len: 0,
    };

    let mut previous = new_vec();

    for i in 1..=seq1.len() {
        let mut current = new_vec();

        for j in 1..=seq2.len() {
            if seq1[i - 1] == seq2[j - 1] {
                current[j] = previous[j - 1] + 1;
                if current[j] > result.len {
                    result.len = current[j];
                    result.sec1_start = i - result.len;
                    result.sec2_start = j - result.len;
                }
            } else {
                current[j] = 0;
            }
        }

        previous = current;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overlapping() {
        assert_eq!(
            longest_common_coherent_subsequence(
                vec!["a", "b", "c", "d", "f"],
                vec!["a", "c", "d", "e", "f"],
            ),
            LongestCommonCoherentSubsequence {
                sec1_start: 2,
                sec2_start: 1,
                len: 2,
            },
        );
    }

    #[test]
    fn test_one_contains_other() {
        assert_eq!(
            longest_common_coherent_subsequence(
                vec!["a", "b", "c", "d", "e"],
                vec!["b", "c", "d"],
            ),
            LongestCommonCoherentSubsequence {
                sec1_start: 1,
                sec2_start: 0,
                len: 3,
            },
        );
        assert_eq!(
            longest_common_coherent_subsequence(
                vec!["b", "c", "d"],
                vec!["a", "b", "c", "d", "e"],
            ),
            LongestCommonCoherentSubsequence {
                sec1_start: 0,
                sec2_start: 1,
                len: 3,
            },
        );
    }

    #[test]
    fn test_one_extends_other() {
        assert_eq!(
            longest_common_coherent_subsequence(
                vec!["a", "b", "c", "d", "e"],
                vec!["a", "b", "c"],
            ),
            LongestCommonCoherentSubsequence {
                sec1_start: 0,
                sec2_start: 0,
                len: 3,
            },
        );
        assert_eq!(
            longest_common_coherent_subsequence(
                vec!["a", "b", "c"],
                vec!["a", "b", "c", "d", "e"],
            ),
            LongestCommonCoherentSubsequence {
                sec1_start: 0,
                sec2_start: 0,
                len: 3,
            },
        );
        assert_eq!(
            longest_common_coherent_subsequence(
                vec!["a", "b", "c", "d", "e"],
                vec!["c", "d", "e"],
            ),
            LongestCommonCoherentSubsequence {
                sec1_start: 2,
                sec2_start: 0,
                len: 3,
            },
        );
        assert_eq!(
            longest_common_coherent_subsequence(
                vec!["c", "d", "e"],
                vec!["a", "b", "c", "d", "e"],
            ),
            LongestCommonCoherentSubsequence {
                sec1_start: 0,
                sec2_start: 2,
                len: 3,
            },
        );
    }
}