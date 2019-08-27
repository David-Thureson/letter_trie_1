#![allow(dead_code)]

// use std::rand::{task_rng, Rng};
use regex::Regex;
use std::collections::{BTreeMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

type PrefixEntry = BTreeMap<String, NextStep>;
type PrefixMap = BTreeMap<String, PrefixEntry>;
type SequenceMap = BTreeMap<usize, PrefixMap>;

const START: &str = "[";
const END: &str = "]";
const CHANCE_TO_USE_DEPTH: f64 = 1.0;
const MAX_WORD_LENGTH: usize = 16;

#[derive(Debug)]
struct NextStep {
    value: String,
    count: usize,
    share: f64,
    range_start: f64,
    range_end: f64,
}

/// Given a small list of real words in some language, generate a large number of mostly fake words that resemble the real ones.
///
/// This is used for testing large tries while avoiding any licensing issues. In English at least, lists of over 5,000 common
/// words tend to be available only under a license and couldn't be posted publicly on GitHub.
///
/// For a valid test the generated words should follow the patterns of the real words. For instance, going by the list of the
/// 3,000 most common English words:
///	- About 11.3% of words start with "s" while only 4.8% of words start with "b".
/// - Given only that the last letter in a partially-formed word is "b", about 17% of the time that will be followed by an "e" and about 3% of the time this "b" will be the last letter of the word.
/// - Given only that the last three letters in a partially-formed word are "ome", 38% of the time that's the end of the word.
///
/// By randomly generating words while still following these patterns, we end up with as many words as we like that resemble
/// the target language such as, for English:
/// - gardench
/// - excelline
/// - illutiful
/// - weatest
/// - passignal
/// - borror
/// - nearan
/// - spinkind
/// - seemer
/// - incomply
/// - welve
/// - conoministrator
/// - remanager
/// - operical
///
/// The `depth` argument is the highest number of letters to use when deciding what letter to add to a partially-formed word.
/// A value of 1 would mean using only the final letter. A lower number leads to generated words that look more random and less
/// like the example words. A higher number leads to generated words that are only slight variations on the example
/// words, and it may not be possible to generate enough new words. The test files in this project such as
/// "fake_words_400_000_sorted.txt" were generated with a depth of 3, as were the sample words in the list above.
///
/// It would be simpler to generate words completely randomly with each letter, pair, or triplet having an equal chance of being
/// followed by any letter except for some fixed chance of the word ending at that point. However, this would lead to a much
/// broader trie with far more nodes for the same number of words, so we'd expect it to performa differently from a trie that
/// was made from real words.
///
/// Note that the list of generated words may contain any number of the original seed words depending on the luck of the draw.
///
/// Arguments:
/// - `example_words`: A list of real words in the target language.
/// - `target_count`: The number of words to generate. This can be much larger than the number of words in `example_words`.
/// - `depth`: The number of letters in a partially-formed word to use when choosing the next letter.
///
/// # Examples
///
/// ```rust
/// use letter_trie::*;
///
/// let source_filename = "english_words_3_000.txt";
/// let source_word_count = 3_000;
///
///	// A million words would be fine but it's a smaller number here since this doc test will be
/// // run repeatedly.
/// let generated_word_count = 50_000;
/// let depth = 3;
///
/// let example_words: Vec<String> = words_from_file(&source_filename);
/// assert_eq!(example_words.len(), source_word_count);
///
/// let generated_words = generate_words(&example_words, generated_word_count, depth);
/// assert_eq!(generated_words.len(), generated_word_count);
/// ```
pub fn generate_words(
    example_words: &[String],
    target_count: usize,
    max_depth: usize,
) -> Vec<String> {
    let sequence_map = make_sequence_map(example_words, max_depth);
    // print_sequence_map(&sequence_map);
    let mut set: HashSet<String> = HashSet::new();
    while set.len() < target_count {
        let mut word = String::from(START);
        while add_to_word(&sequence_map, &mut word) {}
        let final_word: String = (&word[1..]).to_lowercase().to_owned();
        //if !example_sequences.contains(&final_word)
        if !final_word.is_empty() && final_word.len() <= MAX_WORD_LENGTH {
            let set_len = set.len();
            if set_len % 1_000 == 0 {
                println!("[{}] {}", set.len(), &final_word);
            }
            set.insert(final_word);
        }
    }
    let v: Vec<String> = set.drain().collect::<Vec<String>>();
    // println!("{:#?}", v);
    println!("{:#?}", v.len());
    v
}

fn add_to_word(sequence_map: &SequenceMap, word: &mut String) -> bool {
    let word_len = word.len();
    let mut depth = word_len;
    // let mut depth = task_rng().gen_range(1, word_len + 1);
    while depth >= 1 {
        // if rand::random::<f64>() < CHANCE_TO_USE_DEPTH {
        if let Some(prefix_map) = sequence_map.get(&depth) {
            let prefix = &word[word_len - depth..].to_owned();
            if let Some(prefix_entry) = prefix_map.get(prefix) {
                let next_step_value = random_weighted_value(&prefix_entry);
                if next_step_value == END {
                    return false;
                } else {
                    *word = format!("{}{}", &word, next_step_value);
                    return true;
                }
            }
        }
        // }
        depth -= 1;
    }
    false
}

fn random_weighted_value(prefix_entry: &PrefixEntry) -> String {
    let r = rand::random::<f64>();
    let next_step = prefix_entry
        .values()
        .find(|x| r >= x.range_start && r < x.range_end)
        .unwrap();
    next_step.value.to_owned()
}

fn make_sequence_map(example_words: &[String], max_depth: usize) -> SequenceMap {
    let regex = Regex::new(r"^[a-z]+$").unwrap();
    // The special characters indicating the beginning and end of a word must not be characters that can be found
    // in a word.
    debug_assert!(!regex.is_match(START));
    debug_assert!(!regex.is_match(END));

    let mut sequence_map = SequenceMap::new();

    for depth in 1..=max_depth {
        let prefix_map = sequence_map.entry(depth).or_insert_with(PrefixMap::new);
        for example in example_words.iter().map(|x| x.trim().to_lowercase()) {
            if regex.is_match(&example) {
                let word = format!("{}{}{}", START, example, END);
                let last_i: isize = (word.len() as isize - depth as isize) - 1;
                if last_i >= 0 {
                    for i in 0..=(last_i as usize) {
                        let prefix = word[i..i + depth].to_owned();
                        let prefix_entry =
                            prefix_map.entry(prefix).or_insert_with(PrefixEntry::new);
                        let next_step_value = word[i + depth..=i + depth].to_owned();
                        let mut next_step =
                            prefix_entry
                                .entry(next_step_value.clone())
                                .or_insert(NextStep {
                                    value: next_step_value,
                                    count: 0,
                                    share: 0.0,
                                    range_start: 0.0,
                                    range_end: 0.0,
                                });
                        next_step.count += 1;
                    }
                }
            }
        }
        for prefix_entry in prefix_map.values_mut() {
            let count_sum = &prefix_entry
                .values()
                .map(|next_step| next_step.count as f64)
                .sum::<f64>();
            let mut range_start = 0.0;
            for mut next_step in prefix_entry.values_mut() {
                let share = next_step.count as f64 / count_sum;
                next_step.share = share;
                next_step.range_start = range_start;
                next_step.range_end = range_start + share;
                range_start += share;
            }
        }
    }
    sequence_map
}

fn print_sequence_map(sequence_map: &SequenceMap) {
    for depth in sequence_map.keys() {
        println!("\nDepth = {}\n", depth);
        let prefix_map = &sequence_map[depth];
        for prefix in prefix_map.keys() {
            println!("  {} ->", prefix);
            for next_step in prefix_map[prefix].values() {
                // println!("    {}: count = {}", next_step.value, next_step.count);
                println!("    {:?}", next_step);
            }
        }
    }
}

/// Given a filename, create a Vec<String> where each entry is one word.
/// This assumes that there is at most one word per line in the file.
///
/// # Panics
///
/// This will fail if the file does not exist or can't be opened for reading.
pub fn words_from_file(filename: &str) -> Vec<String> {
    // The None means don't check the number of words found in the file.
    words_from_file_test(filename, None)
}

/// Given a filename, create a Vec<String> where each entry is one word.
/// This assumes that there is at most one word per line in the file.
///
/// # Panics
///
/// This will fail if the file does not exist or can't be opened for reading.
///
/// It will also fail with an assertion error if `expected_word_count` has a value and doesn't match the
/// number of words found in the file.
pub fn words_from_file_test(filename: &str, expected_word_count: Option<usize>) -> Vec<String> {
    let file = File::open(filename).unwrap();
    let mut v: Vec<String> = vec![];
    for line in BufReader::new(file).lines() {
        let line = line.unwrap();
        let line = line.trim();
        if !line.is_empty() {
            v.push(line.to_string());
        }
    }
    if let Some(exp_word_count) = expected_word_count {
        assert_eq!(v.len(), exp_word_count);
    }
    v
}

pub fn file_from_lines(filename: &str, lines: &[String]) {
    let mut file = File::create(filename).expect("Error creating file.");
    for line in lines {
        writeln!(file, "{}", line).expect("Error writing a line.");
    }
}
