// Enables the use of Weak::strong_count() and Weak::weak_count().
#![feature(weak_counts)]
#![allow(clippy::new_without_default)]
#![feature(test)]
extern crate test;

#[macro_use]
extern crate lazy_static;

use std::collections::HashSet;
use std::fmt::Debug;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::Mutex;
use std::time::Instant;

pub mod base_letter_trie;
pub use base_letter_trie::BaseLetterTrie;
pub mod no_parent_letter_trie;
pub use no_parent_letter_trie::NoParentLetterTrie;
pub mod util;
pub use util::*;

const FILENAME_SMALL_SORTED: &str = "words_9_sorted.txt";
const FILENAME_SMALL_UNSORTED: &str = "words_9_unsorted.txt";
const FILENAME_MEDIUM_SORTED: &str = "words_10_000_sorted.txt";
const FILENAME_MEDIUM_UNSORTED: &str = "words_10_000_unsorted.txt";
const FILENAME_LARGE_SORTED: &str = "words_584_983_sorted.txt";
const FILENAME_LARGE_UNSORTED: &str = "words_584_983_unsorted.txt";
const FILENAME_GOOD_WORDS: &str = "test_good_words.txt";
const FILENAME_NON_WORDS: &str = "test_non_words.txt";
const USE_CHAR_GET_COUNTER: bool = false;

const DEBUG_TRIE_MAX_DEPTH: usize = 1000;
const DEBUG_TRIE_MAX_CHILDREN: usize = 1000;

const LABEL_STEP_OVERALL: &str = "overall load";
const LABEL_STEP_READ_FILE: &str = "read file";
const LABEL_STEP_MAKE_VECTOR: &str = "make_vector";
const LABEL_STEP_SORT_VECTOR: &str = "sort_vector";
const LABEL_STEP_READ_AND_VECTOR: &str = "make vector from file";
const LABEL_STEP_LOAD_FROM_VEC: &str = "load from vector";

/// A letter trie (https://www.geeksforgeeks.org/trie-insert-and-search) with implementations that use different
/// approaches for parent and child links but otherwise work the same.
///
/// One use of such a trie is to play millions of rounds of a word game like Scrabble or Boggle. In either game there
/// is a branching set of possible letter sequences given a set of seven tiles or sixteen dice. The trie allows one
/// to work letter-by-letter through the possible sequences while stepping node-by-node through the trie at the same
/// time. It may be possible to cut the process short without either running out of tiles/dice or reaching a leaf
/// node in the trie. For instance, we may be tracking the best score reached so far and a given node could be
/// annotated with the highest score that could possibly be reached in its subtree. If this highest possible score
/// is less than or equal to the best score we've already found, there's no point in going into that subtree or
/// (which is saying the same thing) in trying to add another tile or die to the word we've been forming.
///
/// A trie of even half a million words in a given language (with probably fewer than 1.5 million nodes) is still going
/// to be orders of magnitude smaller than the possible sequences of letters found in one throw of the Boggle dice.
/// This means that even if we don't cut the search short because of the best possible score in a subtree, we're
/// still in most cases going to run out of trie before we run out of sequences of dice.
pub trait LetterTrie {
    /// Create a trie from words in a text file.
    ///
    /// The text file may contain up to one word per line. The words may be upper- or lowercase and
    /// blank lines and whitespace before or after the words will be ignored. Duplicate words will also be
    /// ignored.
    ///
    /// # Errors
    ///
    /// This will produce an incorrect trie if the file contains lines with more than one word.
    ///
    /// This may crash or produce an incorrect trie if all three of these conditions are met:
    /// - The words in the file are not sorted at least by their first letter (subsequent letters don't matter).
    /// - `is_sorted` is incorrectly set to `true`.
    /// - The load method uses an optimization that relies on the words being sorted by their first letter. Currently the only such load method is `LoadMethod::ContinuousParallel`.
    ///
    /// # Panics
    ///
    /// Panics if the file does not exist or can't be opened for reading.
    fn from_file(filename: &str, is_sorted: bool, load_method: &LoadMethod) -> Self;

    /// Create a trie from words in a text file, optionally displaying elapsed time for each step.
    ///
    /// The text file may contain up to one word per line. The words may be upper- or lowercase and
    /// blank lines and whitespace before or after the words will be ignored. Duplicate words will also be
    /// ignored.
    ///
    /// # Errors
    ///
    /// This will produce an incorrect trie if the file contains lines with more than one word.
    ///
    /// This may crash or produce an incorrect trie if all three of these conditions are met:
    /// - The words in the file are not sorted at least by their first letter (subsequent letters don't matter).
    /// - `is_sorted` is incorrectly set to `true`.
    /// - The load method uses an optimization that relies on the words being sorted by their first letter. Currently the only such load method is `LoadMethod::ContinuousParallel`.
    ///
    /// # Panics
    ///
    /// Panics if the file does not exist or can't be opened for reading.
    fn from_file_test(
        filename: &str,
        is_sorted: bool,
        load_method: &LoadMethod,
        opt: &DisplayDetailOptions,
    ) -> Self;

    /// Given a word or a partial word, find the corresponding node in the trie if it exists.
    fn find(&self, prefix: &str) -> Option<FixedNode>;

    /// For testing or debugging, create a FixedNode from the root node of a trie.
    fn to_fixed_node(&self) -> FixedNode;

    /// Print one line of information about the root node of a trie.
    ///
    /// This includes things like the number of nodes and words in the trie and the maximum height.
    fn print_root(&self) {
        println!("{:?}", self.to_fixed_node());
    }

    /// Print information about the root node of a trie over multiple lines.
    ///
    /// This includes things like the number of nodes and words in the trie and the maximum height.
    fn print_root_alt(&self) {
        println!("{:#?}", self.to_fixed_node());
    }
}

/// Choice of the collection of words to load in the letter trie.
///
/// Whether the words are sorted in the collection may affect the speed of loading the trie depending on the
/// chosen LoadMethod but the resulting trie will be identical either way.
#[derive(Debug)]
pub enum Dataset {
    /// Small file with nine sorted English words leading to a trie with 26 nodes and a maximum height of 9.
    TestSmallSorted,
    /// Small file with nine unsorted English words leading to a trie with 26 nodes and a maximum height of 9.
    TestSmallUnsorted,
    /// Medium file with 10,000 sorted non-English words leading to a trie with 19,089 nodes and a maximum height of 16.
    TestMediumSorted,
    /// Medium file with 10,000 unsorted non-English words leading to a trie with 19,089 nodes and a maximum height of 16.
    TestMediumUnsorted,
    /// Large file with 584,983 sorted non-English words leading to a trie with 1,143,413 nodes and a maximum height of 16.
    TestLargeSorted,
    /// Large file with 584,983 unsorted non-English words leading to a trie with 1,143,413 nodes and a maximum height of 16.
    TestLargeUnsorted,
}

impl Dataset {
    /// Get the path to a file with a set of words for testing.
    ///
    /// # Examples
    ///
    /// Get the path to a file that has 10,000 words.
    ///
    /// ```rust
    /// use letter_trie::*;
    ///
    /// let filename: &str = Dataset::TestMediumSorted.filename();
    /// ```
    pub fn filename(&self) -> &str {
        match self {
            Dataset::TestSmallSorted => FILENAME_SMALL_SORTED,
            Dataset::TestSmallUnsorted => FILENAME_SMALL_UNSORTED,
            Dataset::TestMediumSorted => FILENAME_MEDIUM_SORTED,
            Dataset::TestMediumUnsorted => FILENAME_MEDIUM_UNSORTED,
            Dataset::TestLargeSorted => FILENAME_LARGE_SORTED,
            Dataset::TestLargeUnsorted => FILENAME_LARGE_UNSORTED,
        }
    }

    /// Returns true if the dataset is already in alphabetical order.
    ///
    /// # Examples
    ///
    /// Get the path to a file that has 10,000 words.
    ///
    /// ```rust
    /// use letter_trie::*;
    ///
    /// let is_sorted: bool = Dataset::TestLargeUnsorted.is_sorted();
    /// assert_eq!(false, is_sorted);
    /// ```
    pub fn is_sorted(&self) -> bool {
        match self {
            Dataset::TestSmallSorted | Dataset::TestMediumSorted | Dataset::TestLargeSorted => true,
            Dataset::TestSmallUnsorted
            | Dataset::TestMediumUnsorted
            | Dataset::TestLargeUnsorted => false,
        }
    }
}

/// The choice of implementation of LetterTrie.
#[derive(Debug)]
pub enum LetterTrieType {
    /// The baseline implementation using Rc<RefCell<Node>> for child links and Weak<RefCell<Node>> for parent links.
    Base,
    /// A stripped-down implementation with no parent links and with direct ownership of child nodes.
    NoParent,
}

/// The method the LetterTrie will use to load words from a text file.
#[derive(Debug, PartialEq)]
pub enum LoadMethod {
    /// Read the whole file into memory, create a vector of words, then fill the trie.
    ReadVecFill,
    /// Read the file into a vector in one step, then fill the trie.
    VecFill,
    /// Build the trie while reading lines from the file.
    Continuous,
    /// Build the trie by evaluating the set of words for each starting letter in its own thread.
    ///
    /// Read lines from the file, and as soon as all of the words for each starting letter have been read spawn affect
    /// thread to build a trie for that starting letter while continuing to read from the file in the first thread.
    /// As each thread finishes building its trie, merge that trie into the main trie.
    ContinuousParallel,
}

/// Options for the amount of detail to display while building a trie.
pub struct DisplayDetailOptions {
    /// If true, print the elapsed time for the whole trie build including reading the file.
    pub print_overall_time: bool,
    /// If true, print the elapsed time for each step. The particular steps depend on the chosen LoadMethod.
    pub print_step_time: bool,
    /// The amount of debugging information to print about the trie after it's been built:
    /// - 0: Print nothing
    /// - 1: Print a single line for the trie, the equivalent of `println!("{:?}", trie.to_fixed_node());`.
    /// - 2: Print a multiple lines for the trie, the equivalent of `println!("{:#?}", trie.to_fixed_node());`.
    pub object_detail_level: usize,
    /// The label to be displayed with any debugging information. One easy way to create this string is with a
    /// call to `DisplayDetailOptions::get_test_label()`.
    pub label: String,
}

impl DisplayDetailOptions {
    /// Create a set of options that display nothing while building the trie.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use letter_trie::*;
    ///
    /// let display_opt: DisplayDetailOptions = DisplayDetailOptions::make_no_display();
    /// ```
    pub fn make_no_display() -> Self {
        Self {
            print_overall_time: false,
            print_step_time: false,
            object_detail_level: 0,
            label: "".to_owned(),
        }
    }

    /// Create a set of options that display only the overall time to build the trie.
    /// # Examples
    ///
    /// ```rust
    /// use letter_trie::*;
    ///
    /// let dataset = Dataset::TestMediumUnsorted;
    /// let load_method = LoadMethod::Continuous;
    /// let letter_trie_type = LetterTrieType::NoParent;
    ///
    /// let display_opt: DisplayDetailOptions = DisplayDetailOptions::make_overall_time(
    ///     &dataset,
    ///     &load_method,
    ///     &letter_trie_type);
    ///
    /// let mut trie: BaseLetterTrie = BaseLetterTrie::from_file_test(
    ///     &dataset.filename(),
    ///     dataset.is_sorted(),
    ///     &load_method,
    ///     &display_opt);
    /// ```
    pub fn make_overall_time(
        dataset: &Dataset,
        load_method: &LoadMethod,
        letter_trie_type: &LetterTrieType,
    ) -> Self {
        Self {
            print_overall_time: true,
            print_step_time: false,
            object_detail_level: 0,
            label: Self::get_test_label(&dataset, &load_method, &letter_trie_type),
        }
    }

    /// Create a set of options that display the overall time to build the trie as well as the time for each step.
    ///
    /// At the end, if the trie is small enough it will be displayed in its entirety, otherwise only the root node.
    /// # Examples
    ///
    /// ```rust
    /// use letter_trie::*;
    ///
    /// let dataset = Dataset::TestLargeSorted;
    /// let load_method = LoadMethod::ContinuousParallel;
    /// let letter_trie_type = LetterTrieType::Base;
    ///
    /// let display_opt: DisplayDetailOptions = DisplayDetailOptions::make_overall_time(
    ///     &dataset,
    ///     &load_method,
    ///     &letter_trie_type);
    ///
    /// let mut trie: BaseLetterTrie = BaseLetterTrie::from_file_test(
    ///     &dataset.filename(),
    ///     dataset.is_sorted(),
    ///     &load_method,
    ///     &display_opt);
    /// ```
    pub fn make_moderate(
        dataset: &Dataset,
        load_method: &LoadMethod,
        letter_trie_type: &LetterTrieType,
    ) -> Self {
        Self {
            print_overall_time: true,
            print_step_time: true,
            object_detail_level: match dataset {
                Dataset::TestSmallSorted | Dataset::TestSmallUnsorted => 2,
                _ => 1,
            },
            label: Self::get_test_label(&dataset, &load_method, &letter_trie_type),
        }
    }

    /// Create the label to be displayed during the trie build process.
    ///
    /// The label shows the chosen dataset and build method.
    ///
    /// Usually it's not necessary to call this function directly since it's handled in the call to
    /// `DisplayDetailOptions::make_moderate()` and related functions.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use letter_trie::*;
    ///
    /// let dataset = Dataset::TestSmallSorted;
    /// let load_method = LoadMethod::Continuous;
    /// let letter_trie_type = LetterTrieType::Base;
    ///
    /// let label: String = DisplayDetailOptions::get_test_label(
    ///     &dataset,
    ///     &load_method,
    ///     &letter_trie_type);
    /// ```
    pub fn get_test_label(
        dataset: &Dataset,
        load_method: &LoadMethod,
        letter_trie_type: &LetterTrieType,
    ) -> String {
        format!("{:?}; {:?}; {:?}", dataset, load_method, letter_trie_type).to_owned()
    }
}

/// A concrete copy of a real trie node with all owned data, used for testing and debugging.
///
/// It's a way for various implementations of LetterTrie to describe a given node in a simple format without references
/// regardless of how the nodes are really structured. This makes it easy to compare a node against an expected node
/// so it's used in the unit tests. It also allows us to confirm that corresponding nodes between different
/// implementations of the trie, or tries built with different approaches, ultimately resolve to the same values.
/// # Examples
///
/// In a unit test, create a large trie using a particular implementation of LetterTrie and a particular building method,
/// then confirm that the root node has all of the expected values.
///
/// ```rust
/// use letter_trie::*;
///
/// let dataset = Dataset::TestLargeUnsorted;
/// let trie: BaseLetterTrie = BaseLetterTrie::from_file(
///     &dataset.filename(),
///     dataset.is_sorted(),
///     &LoadMethod::Continuous,
/// );
///
/// assert_eq!(
///     trie.to_fixed_node(),
///     FixedNode {
///         c: ' ',
///         prefix: "".to_owned(),
///         depth: 0,
///         is_word: false,
///         child_count: 26,
///         node_count: 1_143_413,
///         word_count: 584_978,
///         height: 16,
///     }
/// );
/// ```
///
/// Create tries from the same list of words (but in different orders) and the same implementations but
/// using two different build methods, then confirm that the tries are effectively identical.
///
/// ```rust
/// use letter_trie::*;
///
/// let dataset = Dataset::TestSmallSorted;
/// let trie_1: BaseLetterTrie = BaseLetterTrie::from_file(
///     &dataset.filename(),
///     dataset.is_sorted(),
///     &LoadMethod::ContinuousParallel,
/// );
///
/// let dataset = Dataset::TestSmallUnsorted;
/// let trie_2: BaseLetterTrie = BaseLetterTrie::from_file(
///     &dataset.filename(),
///     dataset.is_sorted(),
///     &LoadMethod::ReadVecFill,
/// );
///
/// // Confirm that the tries' root nodes are equivalent.
/// assert_eq!(trie_1.to_fixed_node(), trie_2.to_fixed_node());
///
/// // Confirm that certain words are found in both tries and at equivalent nodes in the tries,
/// // or that they're not found in either trie.
/// for word in vec!["creature", "create", "azure", "notfound", "cross", "cre", "an", "and"] {
///     let fixed_node_1: Option<FixedNode> = trie_1.find(word);
///     let fixed_node_2: Option<FixedNode> = trie_2.find(word);
///     assert_eq!(fixed_node_1, fixed_node_2);
/// }
///
/// // Confirm that every node in the first trie matches every corresponding node in the second
/// // trie.
/// let iter_1 = trie_1.iter_breadth_first();
/// let iter_2 = trie_2.iter_breadth_first();
/// for (fixed_node_1, fixed_node_2) in iter_1.zip(iter_2) {
///     assert_eq!(fixed_node_1, fixed_node_2);
/// }
/// ```
#[derive(Debug, PartialEq)]
pub struct FixedNode {
    pub c: char,
    pub prefix: String,
    pub depth: usize,
    pub is_word: bool,
    pub child_count: usize,
    pub node_count: usize,
    pub word_count: usize,
    pub height: usize,
}

//
lazy_static! {
    static ref CHAR_GET_COUNTER: Mutex<CharGetCounter> = Mutex::new(CharGetCounter {
        hit_count: 0,
        miss_count: 0
    });
}

/// A counter to keep track of the node hits and misses while building a trie from a list of words.
/// - hit = Starting from a given node, we found a child node corresponding to the next letter of the word.
/// - miss = We didn't find such a child node and thus created one.
///
/// These results can influence how we go about speeding up the build. In the large word list with 584,983 words
/// leading to 1,143,413 nodes we get a hit about 82% of the time.
#[derive(Debug)]
pub struct CharGetCounter {
    hit_count: usize,
    miss_count: usize,
}

impl CharGetCounter {
    /// Set the counters to zero at the start of a trie build.
    pub fn reset() {
        let mut counter = CHAR_GET_COUNTER.lock().unwrap();
        counter.hit_count = 0;
        counter.miss_count = 0;
    }

    /// Record a single hit or miss.
    pub fn record(is_hit: bool) {
        let mut counter = CHAR_GET_COUNTER.lock().unwrap();
        if is_hit {
            counter.hit_count += 1;
        } else {
            counter.miss_count += 1;
        }
    }

    /// View the results.
    pub fn print() {
        let counter = CHAR_GET_COUNTER.lock().unwrap();
        let total_count = counter.hit_count + counter.miss_count;
        if total_count == 0 {
            println!("CharGetCounter: nothing recorded");
        } else {
            let hit_pct = counter.hit_count as f64 / total_count as f64;
            println!(
                "CharGetCounter: hit count = {}; miss count = {}, hit pct = {}",
                format_count(counter.hit_count),
                format_count(counter.miss_count),
                hit_pct
            );
        }
    }

    /// View the results only if we have some results.
    ///
    /// This allows us to turn counting on or off for a particular build process without the calling code
    /// having to know whether it's enabled.
    pub fn print_optional() {
        let total_count: usize;
        {
            // Lock the counter and get the total count in a separate scope so that the counter is unlocked
            // before we call Self::print(). If we didn't do this, we'd still have a lock on CHAR_GET_COUNTER
            // when calling Self::print(). That function would try to get a lock and wait forever.
            let counter = CHAR_GET_COUNTER.lock().unwrap();
            total_count = counter.hit_count + counter.miss_count;
        }
        if total_count > 0 {
            Self::print();
        }
    }
}

/// Given a filename, create a Vec<Vec<char>> which is the most convenient starting point for building a trie
/// from a list of words. This assumes that there is at most one word per line in the file.
fn make_vec_char(filename: &str, opt: &DisplayDetailOptions) -> Vec<Vec<char>> {
    let start = Instant::now();
    let file = File::open(filename).unwrap();
    let mut v: Vec<Vec<char>> = vec![];
    for line in BufReader::new(file).lines() {
        let line = line.unwrap();
        let line = line.trim();
        if !line.is_empty() {
            let vec_char: Vec<char> = line.to_lowercase().chars().collect();
            v.push(vec_char);
        }
    }
    print_elapsed_from_start(
        opt.print_step_time,
        &opt.label,
        LABEL_STEP_READ_AND_VECTOR,
        start,
    );

    if opt.object_detail_level >= 1 {
        println!("\nWord count = {}", v.len());
    }

    v
}

/// Given a filename, create a Vec<String> where each entry is one word.
/// This assumes that there is at most one word per line in the file.
///
/// # Panics
///
/// This will fail if the file does not exist or can't be opened for reading.
pub fn words_from_file(filename: &str) -> Vec<String> {
    let file = File::open(filename).unwrap();
    let mut v: Vec<String> = vec![];
    for line in BufReader::new(file).lines() {
        let line = line.unwrap();
        let line = line.trim();
        if !line.is_empty() {
            v.push(line.to_string());
        }
    }
    v
}

/// For testing, create a vector of 1,000 words that are known to be in the large word list.
///
/// The large word list is the one corresponding to Dataset::TestLargeSorted or Dataset::TestLargeUnsorted.
/// If a trie is built correctly from one of those datasets, it should be possible to find each of these words
/// in the trie by calling either:
/// - LetterTrie::find() which should return Some(FixedNode).
/// - LetterTrie::is_word() which should return true.
///
/// # Panics
///
/// Panics if the file does not exist or can't be opened for reading.
pub fn good_words() -> Vec<String> {
    words_from_file(FILENAME_GOOD_WORDS)
}

/// For testing, create a vector of 1,000 words that are known NOT to be in the large word list.
///
/// The large word list is the one corresponding to Dataset::TestLargeSorted or Dataset::TestLargeUnsorted.
/// If a trie is built correctly from one of those datasets, it should NOT be possible to find each of these
/// words in the trie by calling either:
/// - LetterTrie::find() which should return None.
/// - LetterTrie::is_word() which should return false.
///
/// # Panics
///
/// Panics if the file does not exist or can't be opened for reading.
pub fn non_words() -> Vec<String> {
    words_from_file(FILENAME_NON_WORDS)
}

/// For testing, create a HashSet containing all of the words in the large dataset.
///
/// This is the list of 584,983 non-English words corresponding to Dataset::TestLargeSorted or
/// Dataset::TestLargeUnsorted.
///
/// We can use this to create a baseline benchmark fer finding our 1,000 known good words and our 1,000 known
/// non-words using only the HashSet. This is the test called "base_letter_trie::tests::bench_is_word_hash_set".
/// We can compare the performance of the HashSet to that of searching for the same words in a BaseLetterTrie and
/// a NoParentLetterTrie.
///
/// This is not really a fair test because these letter tries are not intended for fast searching of whole
/// words. If that's all we wanted to do the HashSet would work fine. Instead the idea is to be able to
/// step letter-by-letter through the trie while following some set of possible letter sequences one letter
/// at a time in parallel (see the comments on letter_trie::LetterTrie).
///
/// # Panics
///
/// Panics if the file for the Dataset::TestLargeSorted dataset does not exist or can't be opened for reading.
pub fn large_dataset_words_hash_set() -> HashSet<String> {
    let mut hash_set = HashSet::new();
    for word in words_from_file(Dataset::TestLargeSorted.filename()) {
        hash_set.insert(word);
    }
    hash_set
}
