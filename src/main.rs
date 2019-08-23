#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_assignments)]
// #![allow(unused_mut)]

use letter_trie::*;
use rand::seq::SliceRandom;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::mem;

extern crate typename;

extern crate util;
use util::*;

const FILENAME_GOOD_WORDS: &str = "C:\\Data\\Text\\test_good_words.txt";
const FILENAME_NON_WORDS: &str = "C:\\Data\\Text\\test_non_words.txt";
const USE_CHAR_GET_COUNTER: bool = false;
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
	// create_find_files();
	try_load_words();
	
	
}

fn try_load_words() {
	dbg!(good_words().len());
	dbg!(non_words().len());
	dbg!(large_dataset_words_hash_set().len());
}

fn small_trie() -> BaseLetterTrie {
    BaseLetterTrie::from_file(
        &Dataset::TestSmallSorted.filename(),
        true,
        &LoadMethod::Continuous)
}

fn large_trie() -> BaseLetterTrie {
    BaseLetterTrie::from_file(
        &Dataset::TestLargeSorted.filename(),
        true,
        &LoadMethod::ContinuousParallel)
}

fn try_large_trie() {
	let dataset = Dataset::TestLargeSorted;
	let load_method = LoadMethod::ContinuousParallel;
	let letter_trie_type = LetterTrieType::Base;
    let opt = DisplayDetailOptions::make_moderate(&dataset, &load_method, &letter_trie_type);
    let t = BaseLetterTrie::from_file_test(&dataset.filename(), dataset.is_sorted(), &load_method, &opt);
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
	base_letter_trie::assert_large_root(&t);
	
    print_elapsed(true, fn_name, LABEL_FREEZE, || t.freeze());
    print_elapsed(true, fn_name, LABEL_PRINT_ROOT, || t.print_root_alt());
	base_letter_trie::assert_large_root(&t);
	
    print_elapsed(true, fn_name, LABEL_UNFREEZE, || t.unfreeze());
    print_elapsed(true, fn_name, LABEL_PRINT_ROOT, || t.print_root_alt());
	base_letter_trie::assert_large_root(&t);
}	

fn display_small_trie() {
    let t = BaseLetterTrie::from_file(
        &Dataset::TestSmallUnsorted.filename(),
        false,
        &LoadMethod::Continuous,
    );
    println!("{:#?}", &t);
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

fn try_one_combination(dataset: &Dataset, load_method: &LoadMethod, letter_trie_type: &LetterTrieType) {
    let filename = &dataset.filename();
    let is_sorted = dataset.is_sorted();
    // let opt = DisplayDetailOptions::make_overall_time(dataset, load_method, letter_trie_type);
    let opt = DisplayDetailOptions::make_moderate(dataset, load_method, letter_trie_type);
    if USE_CHAR_GET_COUNTER {
        CharGetCounter::reset();
    }
    match letter_trie_type {
        LetterTrieType::Base => {
            BaseLetterTrie::from_file_test(filename, is_sorted, &load_method, &opt);
        }
		LetterTrieType::NoParent => {
            if is_sorted || *load_method != LoadMethod::ContinuousParallel {
                NoParentLetterTrie::from_file_test(filename, is_sorted, &load_method, &opt);
            }
        }
    };
    if USE_CHAR_GET_COUNTER {
        CharGetCounter::print_optional();
    }
}

fn create_find_files() {
    let content = fs::read_to_string(Dataset::TestLargeSorted.filename()).expect("Error reading file.");
    let source_vec: Vec<&str> = content.split('\n').collect();
	let mut words = vec![];
	let mut non_words = vec![];
	for word in source_vec.iter().step_by(500).take(1_000) {
		words.push(word.to_owned());
		non_words.push(format!("{}q", word));
	}
	
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
