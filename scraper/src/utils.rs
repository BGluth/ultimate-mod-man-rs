use std::cmp::Ordering;

use float_ord::FloatOrd;
use rust_fuzzy_search::fuzzy_compare;

pub(crate) type FuzzySearchScore = FloatOrd<f32>;

#[derive(Debug)]
pub(crate) enum FuzzySearchMatchRes {
    Perfect(usize),

    /// Fuzzy matches sorted by score descending.
    Multiple(Vec<FuzzyMatchedStr>),
    None,
}

#[derive(Clone, Debug)]
pub(crate) struct FuzzyMatchedStr {
    pub(crate) idx: usize,
    pub(crate) score: FuzzySearchScore,
}

impl Eq for FuzzyMatchedStr {}
impl PartialEq for FuzzyMatchedStr {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
    }
}

impl Ord for FuzzyMatchedStr {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score.cmp(&other.score)
    }
}

impl PartialOrd for FuzzyMatchedStr {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub(crate) fn fuzzy_search_strings_and_return_one_or_many_depending_on_perfect_match<
    T: AsRef<str>,
>(
    strs: &[T],
    search_str: &str,
) -> FuzzySearchMatchRes {
    if strs.is_empty() || search_str.is_empty() {
        return FuzzySearchMatchRes::None;
    }

    let mut scored_str_idxs = strs
        .iter()
        .enumerate()
        .map(|(idx, str)| FuzzyMatchedStr {
            idx,
            score: FloatOrd(fuzzy_compare(search_str, str.as_ref())),
        })
        .collect::<Vec<_>>();

    scored_str_idxs.sort();

    let have_a_perfect_match = scored_str_idxs[0].score == FloatOrd(1.0);
    match have_a_perfect_match {
        false => FuzzySearchMatchRes::Multiple(scored_str_idxs),
        true => FuzzySearchMatchRes::Perfect(scored_str_idxs[0].idx),
    }
}
