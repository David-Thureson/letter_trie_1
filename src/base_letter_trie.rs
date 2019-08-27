extern crate test;

use std::cell::RefCell;
use std::cmp;
use std::collections::BTreeMap;
use std::fmt::{self, Debug};
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::rc::{Rc, Weak};
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

use crate::*;

// The Rc pointing to a node should always have a count of one except in special cases where additional references are
// used temporarily to simplify operations like iterating. There will also be an extra strong count when a ParentLink
// is momentarily upgraded.
type ChildLink = Rc<RefCell<Node>>;
// The weak count of the pointer to a node should always equal that node's number of child nodes.
type ParentLink = Weak<RefCell<Node>>;

/// The baseline implementation of a [letter trie]: https://www.geeksforgeeks.org/trie-insert-and-search/ with added
/// references from nodes to their parents to experiment with Rc and RefCell. Other trees use different approaches
/// for parent and child links but otherwise work the same.
pub struct BaseLetterTrie {
    // The root node's character is a single space which doesn't count toward the words represented by the trie.
    root: ChildLink,
}

impl BaseLetterTrie {
    /// Constructor for the letter trie. The root of each trie is the same regardless of what words will be added to
    /// the trie so there are no parameters.
    ///
    /// # Examples
    /// ```rust
    /// let mut trie = letter_trie::BaseLetterTrie::new();
    /// ```
    pub fn new() -> BaseLetterTrie {
        let c = ' ';
        let depth = 0;
        let parent = None;
        let is_word = false;
        let root = BaseLetterTrie::make_child_node_and_link(c, parent, depth, is_word);
        debug_assert!(Self::child_link_has_normal_ref_counts(&root));
        BaseLetterTrie { root }
    }

    // Create an Rc<RefCell<Node>> for a given character.
    fn make_child_node_and_link(
        c: char,
        parent: Option<ParentLink>,
        depth: usize,
        is_word: bool,
    ) -> ChildLink {
        debug_assert!(Self::opt_parent_link_has_normal_ref_counts(&parent));
        let children = BTreeMap::new();
        Rc::new(RefCell::new(Node {
            c,
            depth,
            parent,
            children,
            is_word,
            is_frozen: false,
            node_count: None,
            word_count: None,
            height: None,
        }))
    }

    fn add_word(&self, s: &str) {
        let s = s.trim();
        if !s.is_empty() {
            debug_assert!(!self.is_frozen());
            let v: Vec<char> = s.to_lowercase().chars().collect();
            let v_len = v.len();
            self.add_from_vec_chars(&v, v_len, 0);
        }
    }

    // This is called once for every word, and should be called only on the root.
    pub fn add_from_vec_chars(&self, v: &[char], v_len: usize, char_index: usize) {
        debug_assert!(!self.is_frozen());
        debug_assert!(self.root.borrow().c == ' ');
        if v_len > 0 {
            BaseLetterTrie::add_from_vec_chars_one_char(&self.root, v, v_len, char_index);
        }
    }

    // This is called once for every character in every word.
    fn add_from_vec_chars_one_char(rc: &ChildLink, v: &[char], v_len: usize, char_index: usize) {
        debug_assert!(Self::child_link_has_normal_ref_counts(&rc));
        if char_index < v_len {
            let c = v[char_index];
            let is_word = char_index == v_len - 1;
            let mut root = rc.borrow_mut();
            let child_node_opt = root.children.get(&c);

            if USE_CHAR_GET_COUNTER {
                CharGetCounter::record(child_node_opt.is_some());
            }

            if let Some(child_node_link) = child_node_opt {
                debug_assert!(Self::child_link_has_normal_ref_counts(&child_node_link));
                if is_word {
                    let mut child_node = child_node_link.borrow_mut();
                    child_node.is_word = true;
                }
                BaseLetterTrie::add_from_vec_chars_one_char(
                    &child_node_link,
                    v,
                    v_len,
                    char_index + 1,
                );
            } else {
                debug_assert!(Self::child_link_has_normal_ref_counts(&rc));
                let parent: ParentLink = Rc::downgrade(&rc);
                debug_assert!(Self::parent_link_has_normal_ref_counts(&parent));
                let new_child_link: ChildLink = BaseLetterTrie::make_child_node_and_link(
                    c,
                    Some(parent),
                    root.depth + 1,
                    is_word,
                );
                BaseLetterTrie::add_from_vec_chars_one_char(
                    &new_child_link,
                    v,
                    v_len,
                    char_index + 1,
                );
                root.children.insert(c, new_child_link);
            }
        }
    }

    pub fn merge(&self, other: BaseLetterTrie) {
        let mut this_node = self.root.borrow_mut();
        for other_child_node_link in other.root.borrow().children.values() {
            debug_assert!(Self::child_link_has_normal_ref_counts(
                &other_child_node_link
            ));
            let mut other_child_node = other_child_node_link.borrow_mut();
            let parent: ParentLink = Rc::downgrade(&self.root);
            other_child_node.parent = Some(parent);
            debug_assert!(Self::opt_parent_link_has_normal_ref_counts(
                &other_child_node.parent
            ));
            let c = other_child_node.c;
            this_node
                .children
                .insert(c, Rc::clone(other_child_node_link));
            debug_assert!(Self::child_link_has_normal_ref_counts(
                &other_child_node_link
            ));
        }
    }

    pub fn print_prefixes(&self, prefix_count: usize) -> usize {
        self.root.borrow().print_prefixes(prefix_count)
    }

    pub fn get_words(&self, word_count: usize) -> Vec<String> {
        let mut v: Vec<String> = vec![];
        self.root.borrow().get_words(&mut v, word_count);
        v
    }

    pub fn print_words(&self, word_count: usize) {
        let v = self.get_words(word_count);
        for word in v {
            println!("{}", word);
        }
    }

    fn is_frozen(&self) -> bool {
        self.root.borrow().is_frozen
    }

    pub fn iter_breadth_first(&self) -> BaseLetterTrieIteratorBreadthFirst {
        BaseLetterTrieIteratorBreadthFirst {
            stack: vec![Rc::clone(&self.root)],
        }
    }

    pub fn iter_prefix(&self, prefix: &str) -> BaseLetterTrieIteratorPrefix {
        let prefix: Vec<char> = prefix.to_lowercase().chars().collect();
        let prefix_len = prefix.len();
        BaseLetterTrieIteratorPrefix {
            prefix,
            prefix_len,
            prefix_index: 0,
            rc: Rc::clone(&self.root),
        }
    }

    pub fn freeze(&mut self) {
        self.root.borrow_mut().freeze();
    }

    pub fn unfreeze(&mut self) {
        self.root.borrow_mut().unfreeze();
    }

    fn print(&self, detail_level: usize) {
        match detail_level {
            1 => println!("{:?}", self.to_fixed_node()),
            2 => println!("{:#?}", self.to_fixed_node()),
            _ => (),
        }
    }

    fn load_read_vec_fill(
        &self,
        filename: &str,
        opt: &DisplayDetailOptions,
        expected_word_count: Option<usize>,
    ) {
        println!("{}", filename);
        let start = Instant::now();
        let content = fs::read_to_string(filename).expect("Error reading file.");
        print_elapsed_from_start(opt.print_step_time, &opt.label, LABEL_STEP_READ_FILE, start);

        let start = Instant::now();
        let words: Vec<&str> = content
            .split('\n')
            .map(|x| x.trim())
            .filter(|x| !x.is_empty())
            .collect();
        if let Some(exp_word_count) = expected_word_count {
            assert_eq!(words.len(), exp_word_count);
        }
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

    fn load_vec_fill(
        &self,
        filename: &str,
        opt: &DisplayDetailOptions,
        expected_word_count: Option<usize>,
    ) {
        let start = Instant::now();
        let v = make_vec_char_test(filename, opt, expected_word_count);
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

    fn load_continuous(&self, filename: &str, expected_word_count: Option<usize>) {
        let file = File::open(filename).unwrap();
        let lines = BufReader::new(file)
            .lines()
            .map(|x| x.unwrap().trim().to_owned())
            .filter(|x| !x.is_empty())
            .collect::<Vec<String>>();
        if let Some(exp_word_count) = expected_word_count {
            assert_eq!(lines.len(), exp_word_count);
        }

        for line in lines {
            let vec_char: Vec<char> = line.to_lowercase().chars().collect();
            let v_len = vec_char.len();
            self.add_from_vec_chars(&vec_char, v_len, 0);
        }
    }

    fn load_continuous_parallel_sorted(&self, filename: &str, expected_word_count: Option<usize>) {
        let (tx, rx) = mpsc::channel();

        let file = File::open(filename).unwrap();
        let lines = BufReader::new(file)
            .lines()
            .map(|x| x.unwrap().trim().to_owned())
            .filter(|x| !x.is_empty())
            .collect::<Vec<String>>();
        if let Some(exp_word_count) = expected_word_count {
            assert_eq!(lines.len(), exp_word_count);
        }

        let mut thread_count = 0;
        let mut prev_c = ' ';
        let mut this_vec: Vec<Vec<char>> = vec![];
        for line in lines {
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

        thread_count += Self::create_thread_for_part_of_vec(this_vec, mpsc::Sender::clone(&tx));

        for (received_index, received) in rx.iter().enumerate() {
            self.merge(received);
            if received_index == thread_count - 1 {
                break;
            }
        }
    }

    fn load_parallel_unsorted(
        &self,
        filename: &str,
        opt: &DisplayDetailOptions,
        expected_word_count: Option<usize>,
    ) {
        let mut v = make_vec_char_test(filename, opt, expected_word_count);

        print_elapsed(
            opt.print_step_time,
            &opt.label,
            LABEL_STEP_SORT_VECTOR,
            || v.sort_unstable_by(|a, b| a[0].cmp(&b[0])),
        );

        let (tx, rx) = mpsc::channel();

        let mut thread_count = 0;
        let mut prev_c = ' ';
        let mut this_vec: Vec<Vec<char>> = vec![];
        for vec_char in v {
            let this_c = vec_char[0];
            if this_c != prev_c {
                thread_count +=
                    Self::create_thread_for_part_of_vec(this_vec, mpsc::Sender::clone(&tx));
                this_vec = vec![];
                prev_c = this_c;
            }
            this_vec.push(vec_char.clone());
        }

        thread_count += Self::create_thread_for_part_of_vec(this_vec, mpsc::Sender::clone(&tx));

        for (received_index, received) in rx.iter().enumerate() {
            self.merge(received);
            if received_index == thread_count - 1 {
                break;
            }
        }
    }

    // Returns the number of threads spawned, which will be 1 if there are items in the vector, otherwise 0.
    fn create_thread_for_part_of_vec(v: Vec<Vec<char>>, tx: mpsc::Sender<BaseLetterTrie>) -> usize {
        if !v.is_empty() {
            thread::spawn(move || {
                let t = BaseLetterTrie::new();
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

    pub fn find(&self, prefix: &str) -> Option<FixedNode> {
        let prefix: Vec<char> = prefix.to_lowercase().chars().collect();
        let prefix_len = prefix.len();
        self.root.borrow().find_child(prefix, prefix_len, 0)
    }

    pub fn find_loop(&self, prefix: &str) -> Option<FixedNode> {
        let prefix: Vec<char> = prefix.to_lowercase().chars().collect();
        let prefix_len = prefix.len();
        let mut prefix_index = 0;
        let mut rc = Rc::clone(&self.root);
        loop {
            if prefix_index > prefix_len {
                return None;
            } else {
                if prefix_index == prefix_len {
                    return if rc.borrow().is_word {
                        Some(rc.borrow().to_fixed_node())
                    } else {
                        None
                    };
                }
                let c = prefix[prefix_index];
                let rc_opt = rc.borrow().children.get(&c).map(|x| Rc::clone(x));
                if let Some(rc_next) = rc_opt {
                    rc = rc_next;
                    prefix_index += 1;
                } else {
                    return None;
                }
            }
        }
    }

    pub fn is_word_recursive(&self, prefix: &str) -> bool {
        let prefix: Vec<char> = prefix.to_lowercase().chars().collect();
        let prefix_len = prefix.len();
        self.root.borrow().is_word_child(prefix, prefix_len, 0)
    }

    pub fn is_word_loop(&self, prefix: &str) -> bool {
        let prefix: Vec<char> = prefix.to_lowercase().chars().collect();
        let prefix_len = prefix.len();
        let mut prefix_index = 0;
        let mut rc = Rc::clone(&self.root);
        loop {
            if prefix_index > prefix_len {
                return false;
            } else {
                if prefix_index == prefix_len {
                    return rc.borrow().is_word;
                }
                let c = prefix[prefix_index];
                let rc_opt = rc.borrow().children.get(&c).map(|x| Rc::clone(x));
                if let Some(rc_next) = rc_opt {
                    rc = rc_next;
                    prefix_index += 1;
                } else {
                    return false;
                }
            }
        }
    }

    fn child_link_has_normal_ref_counts(rc: &ChildLink) -> bool {
        // The Rc pointing to a node will normally have a count of one, either from the BaseLetterTrie to the root
        // node or from a parent node to a child node.
        let strong_count = Rc::strong_count(rc);

        // The weak count of the pointer to a node should equal the number of child nodes.
        // let weak_count = Rc::weak_count(rc);

        // dbg!(strong_count);
        // dbg!(weak_count);

        strong_count == 1

        // Don't check against the number of child nodes since this requires a borrow and the ParentLink might
        // already have a mutable borrow against it.
        // let child_node_count = rc.borrow().children.len();
        // weak_count == child_node_count
    }

    fn parent_link_has_normal_ref_counts(weak: &ParentLink) -> bool {
        // This function can't reuse child_link_has_normal_ref_counts because that would mean upgrading weak
        // into an Rc, thus changing the counts.

        // The Rc pointing to a node will normally have a count of one, either from the BaseLetterTrie to the root
        // node or from a parent node to a child node.
        let strong_count = Weak::strong_count(weak);

        // The weak count of the pointer to a node should equal the number of child nodes.
        // let weak_count = Weak::weak_count(weak).unwrap();

        // dbg!(strong_count);
        // dbg!(weak_count);

        strong_count == 1

        // Don't check against the number of child nodes since this requires a borrow and the ParentLink might
        // already have a mutable borrow against it.
        // let child_node_count = weak.upgrade().unwrap().borrow().children.len();
        // weak_count == child_node_count
    }

    fn opt_parent_link_has_normal_ref_counts(weak_opt: &Option<ParentLink>) -> bool {
        if let Some(weak) = weak_opt {
            Self::parent_link_has_normal_ref_counts(&weak)
        } else {
            true
        }
    }
}

impl LetterTrie for BaseLetterTrie {
    fn from_file(filename: &str, is_sorted: bool, load_method: &LoadMethod) -> Self {
        let opt = DisplayDetailOptions::make_no_display();
        Self::from_file_test(filename, is_sorted, load_method, &opt, None)
    }

    fn from_file_test(
        filename: &str,
        is_sorted: bool,
        load_method: &LoadMethod,
        opt: &DisplayDetailOptions,
        expected_word_count: Option<usize>,
    ) -> Self {
        let t = Self::new();
        print_elapsed(
            opt.print_overall_time,
            &opt.label,
            LABEL_STEP_OVERALL,
            || {
                match load_method {
                    LoadMethod::ReadVecFill => {
                        t.load_read_vec_fill(filename, opt, expected_word_count);
                    }
                    LoadMethod::VecFill => {
                        t.load_vec_fill(filename, opt, expected_word_count);
                    }
                    LoadMethod::Continuous => {
                        t.load_continuous(filename, expected_word_count);
                    }
                    LoadMethod::ContinuousParallel => {
                        if is_sorted {
                            t.load_continuous_parallel_sorted(filename, expected_word_count);
                        } else {
                            t.load_parallel_unsorted(filename, opt, expected_word_count);
                        }
                    }
                };
            },
        );
        t
    }

    fn find(&self, prefix: &str) -> Option<FixedNode> {
        let prefix: Vec<char> = prefix.to_lowercase().chars().collect();
        let prefix_len = prefix.len();
        self.root.borrow().find_child(prefix, prefix_len, 0)
    }

    fn to_fixed_node(&self) -> FixedNode {
        self.root.borrow().to_fixed_node()
    }
}

impl Debug for BaseLetterTrie {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.root.borrow().fmt(f)
    }
}

unsafe impl Send for BaseLetterTrie {}

pub struct BaseLetterTrieIteratorBreadthFirst {
    stack: Vec<ChildLink>,
}

impl Iterator for BaseLetterTrieIteratorBreadthFirst {
    type Item = FixedNode;

    fn next(&mut self) -> Option<Self::Item> {
        if self.stack.is_empty() {
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

pub struct BaseLetterTrieIteratorPrefix {
    prefix: Vec<char>,
    prefix_len: usize,
    prefix_index: usize,
    rc: ChildLink,
}

impl Iterator for BaseLetterTrieIteratorPrefix {
    type Item = FixedNode;

    fn next(&mut self) -> Option<Self::Item> {
        println!("BaseLetterTrieIteratorPrefix.next():\n{:#?}", self);
        if self.prefix_index > self.prefix_len {
            None
        } else {
            let fixed_char_node = self.rc.borrow().to_fixed_node();
            if self.prefix_index == self.prefix_len {
                self.prefix_index += 1;
                Some(fixed_char_node)
            } else {
                let c = self.prefix[self.prefix_index];
                let rc_opt = self.rc.borrow().children.get(&c).map(|x| Rc::clone(x));
                if let Some(rc_next) = rc_opt {
                    self.rc = rc_next;
                    self.prefix_index += 1;
                    Some(fixed_char_node)
                } else {
                    None
                }
            }
        }
    }
}

impl Debug for BaseLetterTrieIteratorPrefix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let rc_string = self.rc.borrow().describe_one_line();
        if f.alternate() {
            write!(
                f,
                "BaseLetterTrieIteratorPrefix:\n\tprefix_len = {}\n\tprefix_index = {}\n\trc = {}",
                self.prefix_len, self.prefix_index, &rc_string
            )
        } else {
            write!(
                f,
                "BaseLetterTrieIteratorPrefix: prefix_len = {}, prefix_index = {}, rc = {}",
                self.prefix_len, self.prefix_index, &rc_string
            )
        }
    }
}

struct Node {
    c: char,
    depth: usize,
    parent: Option<ParentLink>,
    children: BTreeMap<char, ChildLink>,
    is_word: bool,
    is_frozen: bool,
    node_count: Option<usize>,
    word_count: Option<usize>,
    height: Option<usize>,
}

impl Node {
    pub fn node_count(&self) -> usize {
        if self.is_frozen {
            self.node_count.unwrap()
        } else {
            let this_count = 1;
            let child_count: usize = self
                .children
                .values()
                .map(|rc| rc.borrow().node_count())
                .sum();
            this_count + child_count
        }
    }

    pub fn word_count(&self) -> usize {
        if self.is_frozen {
            self.word_count.unwrap()
        } else {
            let this_count = if self.is_word { 1 } else { 0 };
            let child_count: usize = self
                .children
                .values()
                .map(|rc| rc.borrow().word_count())
                .sum();
            this_count + child_count
        }
    }

    pub fn height(&self) -> usize {
        if self.is_frozen {
            self.height.unwrap()
        } else {
            let max_child_height: usize = self
                .children
                .values()
                .map(|rc| rc.borrow().height())
                .max()
                .unwrap_or(0);
            max_child_height + 1
        }
    }

    pub fn freeze(&mut self) {
        if !self.is_frozen {
            let mut node_count = 1;
            let mut word_count = if self.is_word { 1 } else { 0 };
            let mut max_child_height = 0;
            for mut child_node in self.children.values().map(|x| x.borrow_mut()) {
                child_node.freeze();
                node_count += child_node.node_count.unwrap();
                word_count += child_node.word_count.unwrap();
                max_child_height = cmp::max(max_child_height, child_node.height.unwrap());
            }
            self.node_count = Some(node_count);
            self.word_count = Some(word_count);
            self.height = Some(max_child_height + 1);
            self.is_frozen = true;
        }
    }

    pub fn unfreeze(&mut self) {
        if self.is_frozen {
            for mut child_node in self.children.values().map(|x| x.borrow_mut()) {
                child_node.unfreeze();
            }
            self.node_count = None;
            self.word_count = None;
            self.height = None;
            self.is_frozen = false;
        }
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
            if let Some(child_rc) = self.children.get(&c) {
                let child_node = child_rc.borrow();
                if prefix_index == prefix_len - 1 {
                    // We've found the root.
                    Some(child_node.to_fixed_node())
                } else {
                    child_node.find_child(prefix, prefix_len, prefix_index + 1)
                }
            } else {
                None
            }
        }
    }

    fn is_word_child(&self, prefix: Vec<char>, prefix_len: usize, prefix_index: usize) -> bool {
        if prefix_index >= prefix_len {
            false
        } else {
            let c = prefix[prefix_index];
            if let Some(child_rc) = self.children.get(&c) {
                let child_node = child_rc.borrow();
                if prefix_index == prefix_len - 1 {
                    // We've found the root.
                    child_node.is_word
                } else {
                    child_node.is_word_child(prefix, prefix_len, prefix_index + 1)
                }
            } else {
                false
            }
        }
    }

    fn to_fixed_node(&self) -> FixedNode {
        FixedNode {
            c: self.c,
            prefix: self.prefix(),
            depth: self.depth,
            is_word: self.is_word,
            child_count: self.children.len(),
            node_count: self.node_count(),
            word_count: self.word_count(),
            height: self.height(),
        }
    }

    pub fn describe_one_line(&self) -> String {
        let prefix_desc = format!(" \"{}\"", self.prefix());
        let is_frozen_desc = if self.is_frozen { " (frozen)" } else { "" };
        let is_word_desc = if self.is_word { " (word)" } else { "" };
        let node_count_desc = format!("; nodes = {}", self.node_count());
        let word_count_desc = format!("; words = {}", self.word_count());
        let depth_desc = format!("; depth = {}", self.depth);
        let height_desc = format!("; height = {}", self.height());
        format!(
            "Node: {:?}{}{}{}{}{}{}{}",
            self.c,
            prefix_desc,
            is_frozen_desc,
            is_word_desc,
            node_count_desc,
            word_count_desc,
            depth_desc,
            height_desc
        )
    }

    pub fn describe_deep(&self, s: &mut String, depth: usize) {
        s.push_str(&format!(
            "{}\n",
            format_indent(depth, &(self.describe_one_line()))
        ));
        if depth < DEBUG_TRIE_MAX_DEPTH {
            for child_node in self
                .children
                .values()
                .map(|x| x.borrow())
                .take(DEBUG_TRIE_MAX_CHILDREN)
            {
                child_node.describe_deep(s, depth + 1);
            }
        }
    }

    pub fn prefix(&self) -> String {
        if let Some(parent_weak) = &self.parent {
            if let Some(parent_rc) = parent_weak.upgrade() {
                let parent_prefix = parent_rc.borrow().prefix();
                return format!("{}{}", parent_prefix, self.c);
            }
        }
        String::from("")
    }

    pub fn print_prefixes(&self, prefix_count: usize) -> usize {
        let mut remaining_prefix_count = prefix_count;
        let mut prefixes_printed = 0;
        for child_node_rc in self.children.values() {
            let child_node = child_node_rc.borrow();
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

    pub fn get_words(&self, v: &mut Vec<String>, word_count: usize) {
        if v.len() >= word_count {
            return;
        }
        if self.is_word {
            v.push(self.prefix());
        }
        if !self.children.is_empty() {
            for (_, child_node_rc) in self.children.iter() {
                child_node_rc.borrow().get_words(v, word_count);
            }
        }
    }
}

impl Debug for Node {
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

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[test]
    fn small_root() {
        let dataset = Dataset::TestSmallUnsorted;
        let t = BaseLetterTrie::from_file(
            &dataset.filename(),
            dataset.is_sorted(),
            &LoadMethod::Continuous,
        );
        assert_small_root(&t.to_fixed_node());
    }

    #[test]
    fn small_prefix_cross() {
        let dataset = Dataset::TestSmallUnsorted;
        let t = BaseLetterTrie::from_file(
            &dataset.filename(),
            dataset.is_sorted(),
            &LoadMethod::Continuous,
        );
        assert_eq!(
            t.find("cross"),
            Some(FixedNode {
                c: 's',
                prefix: "cross".to_owned(),
                depth: 5,
                is_word: true,
                child_count: 1,
                node_count: 3,
                word_count: 2,
                height: 3,
            })
        );
    }

    #[test]
    fn small_prefix_creatu() {
        let dataset = Dataset::TestSmallUnsorted;
        let t = BaseLetterTrie::from_file(
            &dataset.filename(),
            dataset.is_sorted(),
            &LoadMethod::Continuous,
        );
        assert_eq!(
            t.find("creatu"),
            Some(FixedNode {
                c: 'u',
                prefix: "creatu".to_owned(),
                depth: 6,
                is_word: false,
                child_count: 1,
                node_count: 3,
                word_count: 1,
                height: 3,
            })
        );
    }

    #[test]
    fn small_prefix_an() {
        let dataset = Dataset::TestSmallUnsorted;
        let t = BaseLetterTrie::from_file(
            &dataset.filename(),
            dataset.is_sorted(),
            &LoadMethod::Continuous,
        );
        assert_eq!(
            t.find("an"),
            Some(FixedNode {
                c: 'n',
                prefix: "an".to_owned(),
                depth: 2,
                is_word: true,
                child_count: 1,
                node_count: 2,
                word_count: 2,
                height: 2,
            })
        );
    }

    #[test]
    fn small_prefix_c() {
        let dataset = Dataset::TestSmallUnsorted;
        let t = BaseLetterTrie::from_file(
            &dataset.filename(),
            dataset.is_sorted(),
            &LoadMethod::Continuous,
        );
        assert_eq!(
            t.find("c"),
            Some(FixedNode {
                c: 'c',
                prefix: "c".to_owned(),
                depth: 1,
                is_word: false,
                child_count: 1,
                node_count: 20,
                word_count: 6,
                height: 8,
            })
        );
    }

    #[test]
    fn small_prefix_not_found() {
        let dataset = Dataset::TestSmallUnsorted;
        let t = BaseLetterTrie::from_file(
            &dataset.filename(),
            dataset.is_sorted(),
            &LoadMethod::Continuous,
        );
        assert_eq!(t.find("casoun"), None);
    }

    #[test]
    fn large_read_vec_fill_root() {
        let dataset = Dataset::TestLargeUnsorted;
        let t = BaseLetterTrie::from_file(
            &dataset.filename(),
            dataset.is_sorted(),
            &LoadMethod::ReadVecFill,
        );
        assert_large_root(&t.to_fixed_node());
    }

    #[test]
    fn large_vec_fill_root() {
        let dataset = Dataset::TestLargeUnsorted;
        let t = BaseLetterTrie::from_file(
            &dataset.filename(),
            dataset.is_sorted(),
            &LoadMethod::VecFill,
        );
        assert_large_root(&t.to_fixed_node());
    }

    #[test]
    fn large_continuous_root() {
        let dataset = Dataset::TestLargeUnsorted;
        let t = BaseLetterTrie::from_file(
            &dataset.filename(),
            dataset.is_sorted(),
            &LoadMethod::Continuous,
        );
        assert_large_root(&t.to_fixed_node());
    }

    #[test]
    fn large_continuous_parallel_root() {
        let dataset = Dataset::TestLargeSorted;
        let t = BaseLetterTrie::from_file(
            &dataset.filename(),
            dataset.is_sorted(),
            &LoadMethod::ContinuousParallel,
        );
        assert_large_root(&t.to_fixed_node());
    }

    #[test]
    fn is_word_recursive_good_words() {
        let t = large_tree();
        let words = good_words();
        for word in words {
            assert_eq!(true, t.is_word_recursive(&word));
        }
    }

    #[test]
    fn is_word_loop_good_words() {
        let t = large_tree();
        let words = good_words();
        for word in words {
            assert_eq!(true, t.is_word_loop(&word));
        }
    }

    #[test]
    fn is_word_recursive_non_words() {
        let t = large_tree();
        let words = non_words();
        for word in words {
            assert_eq!(false, t.is_word_recursive(&word));
        }
    }

    #[test]
    fn is_word_loop_non_words() {
        let t = large_tree();
        let words = non_words();
        for word in words {
            assert_eq!(false, t.is_word_loop(&word));
        }
    }

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

    #[bench]
    fn bench_load_read_vec_fill(b: &mut Bencher) {
        b.iter(|| {
            let dataset = Dataset::TestMediumSorted;
            BaseLetterTrie::from_file(
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
            BaseLetterTrie::from_file(
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
            BaseLetterTrie::from_file(
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
            BaseLetterTrie::from_file(
                &dataset.filename(),
                dataset.is_sorted(),
                &LoadMethod::ContinuousParallel,
            );
        });
    }

    fn large_tree() -> BaseLetterTrie {
        BaseLetterTrie::from_file(
            Dataset::TestLargeSorted.filename(),
            true,
            &LoadMethod::ContinuousParallel,
        )
    }
}
