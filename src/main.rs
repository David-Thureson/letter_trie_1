#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_assignments)]
// #![allow(unused_mut)]

use letter_trie::*;
use rand::seq::SliceRandom;
use std::collections::BTreeMap;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::iter::FromIterator;
use std::mem;

extern crate typename;

const USE_CHAR_GET_COUNTER: bool = false;

const WORD_COUNT_SMALL: usize = 10;
const WORD_COUNT_MEDIUM: usize = 20_000;
const WORD_COUNT_LARGE: usize = 400_000;
const WORD_COUNT_GOOD: usize = 1_000;
const WORD_COUNT_NON: usize = 1_000;

const FILENAME_MEDIUM_SORTED: &str = "fake_words_20_000_sorted.txt";
const FILENAME_MEDIUM_UNSORTED: &str = "fake_words_20_000_unsorted.txt";
const FILENAME_LARGE_SORTED: &str = "fake_words_400_000_sorted.txt";
const FILENAME_LARGE_UNSORTED: &str = "fake_words_400_000_unsorted.txt";
const FILENAME_GOOD_WORDS: &str = "test_good_words.txt";
const FILENAME_NON_WORDS: &str = "test_non_words.txt";
const FILENAME_ENGLISH_3_000: &str = "english_words_3_000.txt";
const FILENAME_ENGLISH_30: &str = "C:\\Data\\Text\\English words 30.txt";
const FILENAME_ENGLISH_5: &str = "C:\\Data\\Text\\English words 5.txt";

const LABEL_FREEZE: &str = "freeze";
const LABEL_UNFREEZE: &str = "unfreeze";
const LABEL_PRINT_ROOT: &str = "print root";

fn main() {
    println!("\nLetter Trie 5\n");

    // let all_datasets = vec![Dataset::TestSmallSorted, Dataset::TestSmallUnsorted,
    //	Dataset::TestMediumSorted, Dataset::TestMediumUnsorted,
    //	Dataset::TestLargeSorted, Dataset::TestLargeUnsorted];
    let all_datasets = vec![Dataset::TestLargeSorted, Dataset::TestLargeUnsorted];
    // let all_methods = vec![LoadMethod::ReadVecFill, LoadMethod::VecFill, LoadMethod::Continuous, LoadMethod::ContinuousParallel];
    let all_methods = vec![LoadMethod::Continuous, LoadMethod::ContinuousParallel];
    // let all_types = vec![LetterTrieType::Base, LetterTrieType::NoParent, LetterTrieType::MinStruct];
    let all_types = vec![LetterTrieType::Base];

    // try_combinations(&all_datasets, &all_methods, &all_types);
    // display_small_trie();
    // try_large_trie();

    // create_all_shuffled_files(&all_sizes);
    // try_freeze();
    // try_find_loop();
    // try_find_loop_from_iterator();
    // try_find_loop_like_iterator();
    create_find_files();
    // try_load_words();
    // print_node_counts();
    // make_test_files(&FILENAME_ENGLISH_3_000, 3_000, 3);
    // make_test_files(&FILENAME_ENGLISH_30, 5);
    examine_generated_sequences(&FILENAME_ENGLISH_3_000, 3_000, 3);
}

fn examine_generated_sequences(
    source_filename: &str,
    expected_source_word_count: usize,
    max_depth: usize,
) {
    let example_sequences = words_from_file(&source_filename);
    assert_eq!(example_sequences.len(), expected_source_word_count);

    let generated_words = generate_words(&example_sequences, 10, max_depth);
}

fn make_test_files(source_filename: &str, expected_source_word_count: usize, max_depth: usize) {
    let example_sequences = words_from_file(&source_filename);
    assert_eq!(example_sequences.len(), expected_source_word_count);

    let mut words_large = generate_words(&example_sequences, WORD_COUNT_LARGE, max_depth);
    assert_eq!(words_large.len(), WORD_COUNT_LARGE);

    let mut words_medium = Vec::from_iter(words_large[..WORD_COUNT_MEDIUM].iter().cloned());
    assert_eq!(words_medium.len(), WORD_COUNT_MEDIUM);

    file_from_lines(FILENAME_LARGE_UNSORTED, &words_large);
    words_large.sort_unstable();
    file_from_lines(FILENAME_LARGE_SORTED, &words_large);

    file_from_lines(FILENAME_MEDIUM_UNSORTED, &words_medium);
    words_medium.sort_unstable();
    file_from_lines(FILENAME_MEDIUM_SORTED, &words_medium);
}

/*
fn make_test_file(
    source_filename: &str,
    dest_sorted_filename: &str,
    dest_unsorted_filename: &str,
    target_word_count: usize,
    max_depth: usize)
{
    let source_trie = BaseLetterTrie::from_file(
        source_filename,
        false,
        &LoadMethod::Continuous,
    );

    let list = BTreeSet::new();
    (0..target_word_count) {
        let mut word = String::from(" ");
        do {
            let word_len = word.len()
            let prefix: &str = if (word_len < max_depth) {
                &word[..]
            } else {
                &word[(word_len - max_depth)..]
            }
            let current_node = source_trie.find(prefix);
            let current_c = current_node.c;
            let current_
        }
    }
}
*/

fn print_node_counts() {
    // vec![Dataset::TestSmallSorted, Dataset::TestMediumSorted, Dataset::TestLargeSorted]].iter().foreach(|dataset| {
    // 	let t =
    //});
    small_trie().print_root_alt();
    medium_trie().print_root_alt();
    large_trie().print_root_alt();
}

fn try_load_words() {
    dbg!(good_words().len());
    dbg!(non_words().len());
    dbg!(large_dataset_words_hash_set().len());
}

fn small_trie() -> BaseLetterTrie {
    BaseLetterTrie::from_file_test(
        &Dataset::TestSmallSorted.filename(),
        true,
        &LoadMethod::Continuous,
        &DisplayDetailOptions::make_no_display(),
        Some(WORD_COUNT_SMALL),
    )
}

fn medium_trie() -> BaseLetterTrie {
    BaseLetterTrie::from_file_test(
        &Dataset::TestMediumSorted.filename(),
        true,
        &LoadMethod::Continuous,
        &DisplayDetailOptions::make_no_display(),
        Some(WORD_COUNT_MEDIUM),
    )
}

fn large_trie() -> BaseLetterTrie {
    BaseLetterTrie::from_file_test(
        &Dataset::TestLargeSorted.filename(),
        true,
        &LoadMethod::ContinuousParallel,
        &DisplayDetailOptions::make_no_display(),
        Some(WORD_COUNT_LARGE),
    )
}

fn try_large_trie() {
    let dataset = Dataset::TestLargeSorted;
    let load_method = LoadMethod::ContinuousParallel;
    let letter_trie_type = LetterTrieType::Base;
    let opt = DisplayDetailOptions::make_moderate(&dataset, &load_method, &letter_trie_type);
    let t = BaseLetterTrie::from_file_test(
        &dataset.filename(),
        dataset.is_sorted(),
        &load_method,
        &opt,
        Some(WORD_COUNT_LARGE),
    );
    println!("{:#?}", &t.to_fixed_node());
}

fn try_find_loop() {
    let t = small_trie();
    // let word = "creature";
    let word = "and";
    println!("\n{:#?}", t.find(word));
    println!("\n{:#?}", t.find_loop(word));
}

/*
fn try_find_loop_from_iterator() {
    let t = small_trie();
    // let word = "creature";
    let word = "and";
    println!("\n{:#?}", t.find(word));
    println!("\n{:#?}", t.find_loop_from_iterator(word));
}
*/

/*
fn try_find_loop_like_iterator() {
    let t = small_trie();
    let mut prefix = "and";
    println!("\n\"{}\":\n{:#?}", prefix, t.find_loop_like_iterator(prefix));
    prefix = "ands";
    println!("\n\"{}\":\n{:#?}", prefix, t.find_loop_like_iterator(prefix));
    prefix = "creature";
    println!("\n\"{}\":\n{:#?}", prefix, t.find_loop_like_iterator(prefix));
    prefix = "creatu";
    println!("\n\"{}\":\n{:#?}", prefix, t.find_loop_like_iterator(prefix));
}
*/

fn try_freeze() {
    let fn_name = "try_freeze()";
    let mut t = large_trie();
    print_elapsed(true, fn_name, LABEL_PRINT_ROOT, || t.print_root_alt());
    assert_large_root(&t.to_fixed_node());

    print_elapsed(true, fn_name, LABEL_FREEZE, || t.freeze());
    print_elapsed(true, fn_name, LABEL_PRINT_ROOT, || t.print_root_alt());
    assert_large_root(&t.to_fixed_node());

    print_elapsed(true, fn_name, LABEL_UNFREEZE, || t.unfreeze());
    print_elapsed(true, fn_name, LABEL_PRINT_ROOT, || t.print_root_alt());
    assert_large_root(&t.to_fixed_node());
}

fn display_small_trie() {
    println!("{:#?}", &small_trie());
}

fn try_combinations(datasets: &[Dataset], methods: &[LoadMethod], types: &[LetterTrieType]) {
    for one_dataset in datasets {
        for one_method in methods {
            for one_type in types {
                try_one_combination(&one_dataset, &one_method, &one_type);
            }
        }
    }
}

fn try_one_combination(
    dataset: &Dataset,
    load_method: &LoadMethod,
    letter_trie_type: &LetterTrieType,
) {
    let filename = &dataset.filename();
    let is_sorted = dataset.is_sorted();
    // let opt = DisplayDetailOptions::make_overall_time(dataset, load_method, letter_trie_type);
    let opt = DisplayDetailOptions::make_moderate(dataset, load_method, letter_trie_type);
    let expected_word_count = dataset.word_count();
    if USE_CHAR_GET_COUNTER {
        CharGetCounter::reset();
    }
    match letter_trie_type {
        LetterTrieType::Base => {
            BaseLetterTrie::from_file_test(
                filename,
                is_sorted,
                &load_method,
                &opt,
                Some(expected_word_count),
            );
        }
        LetterTrieType::NoParent => {
            if is_sorted || *load_method != LoadMethod::ContinuousParallel {
                NoParentLetterTrie::from_file_test(
                    filename,
                    is_sorted,
                    &load_method,
                    &opt,
                    Some(expected_word_count),
                );
            }
        }
    };
    if USE_CHAR_GET_COUNTER {
        CharGetCounter::print_optional();
    }
}

fn create_find_files() {
    let content =
        fs::read_to_string(Dataset::TestLargeSorted.filename()).expect("Error reading file.");
    let source_vec: Vec<&str> = content.split('\n').collect();
    let mut words = vec![];
    let mut non_words = vec![];
    for word in source_vec.iter().step_by(100).take(WORD_COUNT_GOOD) {
        words.push(word.to_owned());
        non_words.push(format!("{}q", word));
    }
    // If these assertions fail because there are too few words, lower the step_by() value above.
    assert_eq!(words.len(), WORD_COUNT_GOOD);
    assert_eq!(non_words.len(), WORD_COUNT_NON);

    let mut file = File::create(FILENAME_GOOD_WORDS).expect("Error creating target file.");
    for word in words {
        writeln!(file, "{}", word).expect("Error writing a line.");
    }

    file = File::create(FILENAME_NON_WORDS).expect("Error creating target file.");
    for word in non_words {
        writeln!(file, "{}", word).expect("Error writing a line.");
    }
}

/*
fn create_all_shuffled_files(datasets: &Vec<Dataset>) {
    for one_dataset in datasets {
        let source_filename = String::from(one_size.filename());
        let target_filename = source_filename.replace(".txt", "_shuffled.txt");
        create_shuffled_file(&source_filename, &target_filename);
    }
}

fn create_shuffled_file(source_filename: &str, target_filename: &str) {
    let content = fs::read_to_string(source_filename).expect("Error reading file.");
    let mut v: Vec<&str> = content.split('\n').collect();
    v.shuffle(&mut rand::thread_rng());
    let mut file = File::create(target_filename).expect("Error creating target file.");
    for s in v {
        writeln!(file, "{}", s).expect("Error writing a line.");
    }
}
*/

/*
fn try_find() {
    let t = &mut CharExtNode::new();
    t.add_word("creature");
    t.add_word("cross");
    t.add_word("and");
    println!("{:#?}", t);
    let mut found_cross = t.find("cross");
    // let mut found_river = t.find("river");
    altvals!(t.to_fixed_char_node(), t.find("cross"), t.find("creatu"), t.find("an"), t.find("c"));
}

fn try_iterator() {
    let t = &mut CharBTreeNode::new();
    simple_trie(t);
    println!("{:#?}", t);
    for node in t.iter_breadth_first() {
        println!("{:?}", node);
    }
}
*/
