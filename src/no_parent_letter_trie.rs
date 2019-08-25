use std::collections::BTreeMap;
use std::fmt::{self, Debug};
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::Instant;

use crate::*;

pub struct NoParentLetterTrie {
    c: char,
    depth: usize,
    children: BTreeMap<char, Self>,
    is_word: bool,
}

impl NoParentLetterTrie {
    pub fn new() -> Self {
        let c = ' ';
        let depth = 0;
        let is_word = false;
        Self::make_node(c, depth, is_word)
    }

    fn make_node(c: char, depth: usize, is_word: bool) -> Self {
        Self {
            c,
            depth,
            children: BTreeMap::new(),
            is_word,
        }
    }

    fn add_word(&mut self, s: &str) {
        let s = s.trim();
        if !s.is_empty() {
            let v: Vec<char> = s.to_lowercase().chars().collect();
            let v_len = v.len();
            self.add_from_vec_chars(&v, v_len, 0);
        }
    }

    pub fn add_from_vec_chars(&mut self, v: &[char], v_len: usize, char_index: usize) {
        if v_len > 0 {
            self.add_from_vec_chars_one_node(v, v_len, char_index);
        }
    }

    fn add_from_vec_chars_one_node(&mut self, v: &[char], v_len: usize, char_index: usize) {
        if char_index < v_len {
            let c = v[char_index];
            let is_word = char_index == v_len - 1;
            let child_node_opt = self.children.get_mut(&c);
            if let Some(child_node) = child_node_opt {
                if is_word {
                    child_node.is_word = true;
                }
                child_node.add_from_vec_chars_one_node(v, v_len, char_index + 1);
            } else {
                let mut new_child_node = Self::make_node(c, self.depth + 1, is_word);
                new_child_node.add_from_vec_chars_one_node(v, v_len, char_index + 1);
                self.children.insert(c, new_child_node);
            }
        }
    }

    /*
    pub fn merge(&self, other: Self) {
        for other_child_node_key in other.node.children.keys() {
            self.children.insert(other.children.remove(other_child_node_key);
        }
    }

    pub fn get_words(&self, word_count: usize) -> Vec<String> {
        let mut v: Vec<String> = vec![];
        self.get_words_one_node(&mut v, word_count);
        v
    }

    pub fn get_words_one_node(&self, v: &mut Vec<String>, word_count: usize) {
        if v.len() >= word_count {
            return;
        }
        if self.is_word {
            v.push(self.prefix());
        }
        if self.children.len() > 0 {
            for (_, child_node_rc) in self.children.iter() {
                child_node_rc.borrow().get_words(v, word_count);
            }
        }
    }

    pub fn print_words(&self, word_count: usize) {
        let v = self.get_words(word_count);
        for word in v {
            println!("{}", word);
        }
    }

    pub fn iter_breadth_first(&self) -> NoParentLetterTrieIteratorBreadthFirst {
        NoParentLetterTrieIteratorBreadthFirst {
            stack: vec![Rc::clone(&self.node)],
        }
    }
    */

    fn print(&self, detail_level: usize) {
        match detail_level {
            1 => println!("{:?}", self.to_fixed_node()),
            2 => println!("{:#?}", self.to_fixed_node()),
            _ => (),
        }
    }

    fn load_read_vec_fill(&mut self, filename: &str, opt: &DisplayDetailOptions) {
        let start = Instant::now();
        let content = fs::read_to_string(filename).expect("Error reading file.");
        print_elapsed_from_start(opt.print_step_time, &opt.label, LABEL_STEP_READ_FILE, start);

        let start = Instant::now();
        let words: Vec<&str> = content.split('\n').collect();
        print_elapsed_from_start(
            opt.print_step_time,
            &opt.label,
            LABEL_STEP_MAKE_VECTOR,
            start,
        );

        if opt.object_detail_level >= 1 {
            println!("\nWord count = {}", words.len());
        }

        let start = Instant::now();
        for word in words {
            self.add_word(word);
        }
        print_elapsed_from_start(
            opt.print_step_time,
            &opt.label,
            LABEL_STEP_LOAD_FROM_VEC,
            start,
        );

        self.print(opt.object_detail_level);
    }

    fn load_vec_fill(&mut self, filename: &str, opt: &DisplayDetailOptions) {
        let start = Instant::now();
        let v = make_vec_char(filename, opt);
        for vec_char in v {
            let v_len = vec_char.len();
            self.add_from_vec_chars(&vec_char, v_len, 0);
        }
        print_elapsed_from_start(
            opt.print_step_time,
            &opt.label,
            LABEL_STEP_LOAD_FROM_VEC,
            start,
        );
        self.print(opt.object_detail_level);
    }

    fn load_continuous(&mut self, filename: &str) {
        let file = File::open(filename).unwrap();
        for line in BufReader::new(file).lines() {
            let line = line.unwrap();
            let line = line.trim();
            if !line.is_empty() {
                let vec_char: Vec<char> = line.to_lowercase().chars().collect();
                let v_len = vec_char.len();
                self.add_from_vec_chars(&vec_char, v_len, 0);
            }
        }
    }

    fn load_continuous_parallel(&mut self, filename: &str) {
        self.load_continuous(filename);

        /*
        let (tx, rx) = mpsc::channel();

        let file = File::open(filename).unwrap();

        let mut thread_count = 0;
        let mut prev_c = ' ';
        let mut this_vec: Vec<Vec<char>> = vec![];
        for line in BufReader::new(file).lines() {
            let line = line.unwrap();
            let line = line.trim();
            if line.len() > 0 {
                let vec_char: Vec<char> = line.to_lowercase().chars().collect();
                let this_c = vec_char[0];
                if this_c != prev_c {
                    thread_count +=
                        Self::create_thread_for_part_of_vec(this_vec, mpsc::Sender::clone(&tx));
                    this_vec = vec![];
                    prev_c = this_c;
                }
                this_vec.push(vec_char.clone());
            }
        }

        thread_count += Self::create_thread_for_part_of_vec(this_vec, mpsc::Sender::clone(&tx));

        let mut received_count = 0;
        for received in rx {
            //rintln!("\nReceived {:?}", received_t);
            received_count += 1;

            self.merge(received);

            if received_count == thread_count {
                break;
            }
        }
        */
    }

    // Returns the number of threads spawned, which will be 1 if there are items in the vector, otherwise 0.
    /*
    fn create_thread_for_part_of_vec(
        v: Vec<Vec<char>>,
        tx: mpsc::Sender<NoParentLetterTrie>,
    ) -> usize {
        if v.len() > 0 {
            thread::spawn(move || {
                let t = NoParentLetterTrie::new();
                for vec_char in v {
                    let v_len = vec_char.len();
                    t.add_from_vec_chars(&vec_char, v_len, 0);
                }
                tx.send(t).unwrap();
            });
            1
        } else {
            0
        }
    }
    */

    pub fn node_count(&self) -> usize {
        let mut calc_count = 1;
        for child_node in self.children.values() {
            calc_count += child_node.node_count();
        }
        calc_count
    }

    pub fn word_count(&self) -> usize {
        let mut count = if self.is_word { 1 } else { 0 };
        for child_node in self.children.values() {
            count += child_node.word_count();
        }
        count
    }

    pub fn height(&self) -> usize {
        let mut max_child_height = 0;
        for child_node in self.children.values() {
            let child_height = child_node.height();
            if child_height > max_child_height {
                max_child_height = child_height;
            }
        }
        max_child_height + 1
    }

    fn find_child(
        &self,
        prefix: Vec<char>,
        prefix_len: usize,
        prefix_index: usize,
    ) -> Option<FixedNode> {
        if prefix_index >= prefix_len {
            None
        } else {
            let c = prefix[prefix_index];
            if let Some(child_node) = self.children.get(&c) {
                if prefix_index == prefix_len - 1 {
                    // We've found the node.
                    Some(child_node.to_fixed_node())
                } else {
                    child_node.find_child(prefix, prefix_len, prefix_index + 1)
                }
            } else {
                None
            }
        }
    }

    pub fn describe_one_line(&self) -> String {
        let is_word_desc = if self.is_word { " (word)" } else { "" };
        format!("NoParentLetterTrie: {}{}", self.c, is_word_desc)
    }

    pub fn describe_deep(&self, s: &mut String, depth: usize) {
        s.push_str(&format!(
            "{}\n",
            format_indent(depth, &(self.describe_one_line()))
        ));
        if depth < DEBUG_TRIE_MAX_DEPTH {
            for child_node in self.children.values().take(DEBUG_TRIE_MAX_CHILDREN) {
                child_node.describe_deep(s, depth + 1);
            }
        }
    }

    pub fn prefix(&self) -> String {
        String::from("")
    }

    pub fn print_prefixes(&self, prefix_count: usize) -> usize {
        let mut remaining_prefix_count = prefix_count;
        let mut prefixes_printed = 0;
        for child_node in self.children.values() {
            println!("{}", child_node.prefix());
            remaining_prefix_count -= 1;
            if remaining_prefix_count > 0 {
                let one_prefixes_printed = child_node.print_prefixes(remaining_prefix_count);
                remaining_prefix_count -= one_prefixes_printed;
                prefixes_printed += one_prefixes_printed;
            } else {
                break;
            }
        }
        prefixes_printed
    }

    pub fn is_word_recursive(&self, prefix: &str) -> bool {
        let prefix: Vec<char> = prefix.to_lowercase().chars().collect();
        let prefix_len = prefix.len();
        self.is_word_child(prefix, prefix_len, 0)
    }

    fn is_word_child(&self, prefix: Vec<char>, prefix_len: usize, prefix_index: usize) -> bool {
        if prefix_index >= prefix_len {
            false
        } else {
            let c = prefix[prefix_index];
            if let Some(child_node) = self.children.get(&c) {
                if prefix_index == prefix_len - 1 {
                    // We've found the node.
                    child_node.is_word
                } else {
                    child_node.is_word_child(prefix, prefix_len, prefix_index + 1)
                }
            } else {
                false
            }
        }
    }
}

impl LetterTrie for NoParentLetterTrie {
    fn from_file(filename: &str, is_sorted: bool, load_method: &LoadMethod) -> Self {
        let opt = DisplayDetailOptions::make_no_display();
        Self::from_file_test(filename, is_sorted, load_method, &opt)
    }

    fn from_file_test(
        filename: &str,
        _is_sorted: bool,
        load_method: &LoadMethod,
        opt: &DisplayDetailOptions,
    ) -> Self {
        let mut t = Self::new();
        print_elapsed(
            opt.print_overall_time,
            &opt.label,
            LABEL_STEP_OVERALL,
            || {
                match load_method {
                    LoadMethod::ReadVecFill => &t.load_read_vec_fill(filename, opt),
                    LoadMethod::VecFill => &t.load_vec_fill(filename, opt),
                    LoadMethod::Continuous => &t.load_continuous(filename),
                    LoadMethod::ContinuousParallel => &t.load_continuous_parallel(filename),
                };
            },
        );
        t
    }

    fn find(&self, prefix: &str) -> Option<FixedNode> {
        let prefix: Vec<char> = prefix.to_lowercase().chars().collect();
        let prefix_len = prefix.len();
        self.find_child(prefix, prefix_len, 0)
    }

    fn to_fixed_node(&self) -> FixedNode {
        FixedNode {
            c: self.c,
            prefix: "".to_owned(),
            depth: self.depth,
            is_word: self.is_word,
            child_count: self.children.len(),
            node_count: self.node_count(),
            word_count: self.word_count(),
            height: self.height(),
        }
    }
}

// unsafe impl Send for NoParentLetterTrie {}

impl Debug for NoParentLetterTrie {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            let mut s = String::new();
            self.describe_deep(&mut s, 0);
            write!(f, "{}", s)
        } else {
            let s = self.describe_one_line();
            write!(f, "{}", s)
        }
    }
}

/*
pub struct NoParentLetterTrieIteratorBreadthFirst {
    stack: Vec<RcRefNode>,
}

impl Iterator for NoParentLetterTrieIteratorBreadthFirst {
    type Item = FixedNode;

    fn next(&mut self) -> Option<Self::Item> {
        if self.stack.len() == 0 {
            None
        } else {
            let this_rc = self.stack.remove(0);
            let this_node = this_rc.borrow();
            let fixed_char_node = this_node.to_fixed_node();
            for (_, child_node_rc) in this_node.children.iter() {
                self.stack.push(Rc::clone(&child_node_rc));
            }
            Some(fixed_char_node)
        }
    }
}
*/

pub fn assert_small_root(t: &NoParentLetterTrie) {
    assert_eq!(
        t.to_fixed_node(),
        FixedNode {
            c: ' ',
            prefix: "".to_owned(),
            depth: 0,
            is_word: false,
            child_count: 2,
            node_count: 26,
            word_count: 9,
            height: 9,
        }
    );
}

pub fn assert_large_root(t: &NoParentLetterTrie) {
    assert_eq!(
        t.to_fixed_node(),
        FixedNode {
            c: ' ',
            prefix: "".to_owned(),
            depth: 0,
            is_word: false,
            child_count: 26,
            node_count: 1_143_413,
            word_count: 584_978,
            height: 16,
        }
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[test]
    fn small_root() {
        let dataset = Dataset::TestSmallUnsorted;
        let t = NoParentLetterTrie::from_file(
            &dataset.filename(),
            dataset.is_sorted(),
            &LoadMethod::Continuous,
        );
        assert_small_root(&t);
    }

    #[test]
    fn large_read_vec_fill_root() {
        let dataset = Dataset::TestLargeUnsorted;
        let t = NoParentLetterTrie::from_file(
            &dataset.filename(),
            dataset.is_sorted(),
            &LoadMethod::ReadVecFill,
        );
        assert_large_root(&t)
    }

    #[test]
    fn large_vec_fill_root() {
        let dataset = Dataset::TestLargeUnsorted;
        let t = NoParentLetterTrie::from_file(
            &dataset.filename(),
            dataset.is_sorted(),
            &LoadMethod::VecFill,
        );
        assert_large_root(&t)
    }

    #[test]
    fn large_continuous_root() {
        let dataset = Dataset::TestLargeUnsorted;
        let t = NoParentLetterTrie::from_file(
            &dataset.filename(),
            dataset.is_sorted(),
            &LoadMethod::Continuous,
        );
        assert_large_root(&t)
    }

    #[test]
    fn large_continuous_parallel_root() {
        let dataset = Dataset::TestLargeSorted;
        let t = NoParentLetterTrie::from_file(
            &dataset.filename(),
            dataset.is_sorted(),
            &LoadMethod::ContinuousParallel,
        );
        assert_large_root(&t)
    }

    #[test]
    fn is_word_recursive_good_words() {
        let t = large_tree();
        let words = good_words();
        for word in words {
            assert_eq!(true, t.is_word_recursive(&word));
        }
    }

    /*
    #[test]
    fn is_word_loop_good_words() {
        let t = large_tree();
        let words = good_words();
        for word in words {
            assert_eq!(true, t.is_word_loop(&word));
        }
    }
    */

    #[test]
    fn is_word_recursive_non_words() {
        let t = large_tree();
        let words = non_words();
        for word in words {
            assert_eq!(false, t.is_word_recursive(&word));
        }
    }

    /*
    #[test]
    fn is_word_loop_non_words() {
        let t = large_tree();
        let words = non_words();
        for word in words {
            assert_eq!(false, t.is_word_loop(&word));
        }
    }
    */

    #[bench]
    fn bench_is_word_hash_set(b: &mut Bencher) {
        let words = good_words();
        let hash_set = large_dataset_words_hash_set();
        b.iter(|| {
            for word in words.clone() {
                assert_eq!(true, hash_set.contains(&word));
            }
        });
    }

    #[bench]
    fn bench_is_word_recursive(b: &mut Bencher) {
        let words = good_words();
        let t = large_tree();
        b.iter(|| {
            for word in words.clone() {
                assert_eq!(true, t.is_word_recursive(&word));
            }
        });
    }

    /*
    #[bench]
    fn bench_is_word_loop(b: &mut Bencher) {
        let words = good_words();
        let t = large_tree();
        b.iter(|| {
            for word in words.clone() {
                assert_eq!(true, t.is_word_loop(&word));
            }
        });
    }
    */

    #[bench]
    fn bench_load_read_vec_fill(b: &mut Bencher) {
        b.iter(|| {
            let dataset = Dataset::TestMediumSorted;
            NoParentLetterTrie::from_file(
                &dataset.filename(),
                dataset.is_sorted(),
                &LoadMethod::ReadVecFill,
            );
        });
    }

    #[bench]
    fn bench_load_vec_fill(b: &mut Bencher) {
        b.iter(|| {
            let dataset = Dataset::TestMediumSorted;
            NoParentLetterTrie::from_file(
                &dataset.filename(),
                dataset.is_sorted(),
                &LoadMethod::VecFill,
            );
        });
    }

    #[bench]
    fn bench_load_continuous(b: &mut Bencher) {
        b.iter(|| {
            let dataset = Dataset::TestMediumSorted;
            NoParentLetterTrie::from_file(
                &dataset.filename(),
                dataset.is_sorted(),
                &LoadMethod::Continuous,
            );
        });
    }

    #[bench]
    fn bench_load_continuous_parallel(b: &mut Bencher) {
        b.iter(|| {
            let dataset = Dataset::TestMediumSorted;
            NoParentLetterTrie::from_file(
                &dataset.filename(),
                dataset.is_sorted(),
                &LoadMethod::ContinuousParallel,
            );
        });
    }

    fn large_tree() -> NoParentLetterTrie {
        NoParentLetterTrie::from_file(
            Dataset::TestLargeSorted.filename(),
            true,
            &LoadMethod::ContinuousParallel,
        )
    }

    fn words_from_file(filename: &str) -> Vec<String> {
        let file = File::open(filename).unwrap();
        let mut v: Vec<String> = vec![];
        for line in BufReader::new(file).lines() {
            let line = line.unwrap();
            let line = line.trim();
            if line.len() > 0 {
                v.push(line.to_string());
            }
        }
        v
    }

    fn good_words() -> Vec<String> {
        words_from_file(FILENAME_GOOD_WORDS)
    }

    fn non_words() -> Vec<String> {
        words_from_file(FILENAME_NON_WORDS)
    }

    fn large_dataset_words_hash_set() -> HashSet<String> {
        let mut hash_set = HashSet::new();
        for word in words_from_file(Dataset::TestLargeSorted.filename()) {
            hash_set.insert(word);
        }
        hash_set
    }
}
