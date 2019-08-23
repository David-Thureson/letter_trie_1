#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_assignments)]

extern crate util;
use util::*;

use std::cell::RefCell;
use std::cmp;
use std::collections::BTreeMap;
use std::fmt::{self, Debug};
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::rc;
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

use crate::*;

type RcRefNode = Rc<RefCell<Node>>;
type WeakRefNode = rc::Weak<RefCell<Node>>;

pub struct MinStructLetterTrie {
    node: RcRefNode,
}

impl MinStructLetterTrie {
    pub fn new() -> MinStructLetterTrie {
        let c = ' ';
        let node = MinStructLetterTrie::make_node_rc(c);
        MinStructLetterTrie { node }
    }

    fn make_node_rc(c: char) -> RcRefNode {
        let children = BTreeMap::new();
        Rc::new(RefCell::new(Node { c, children }))
    }

    fn add_word(&self, s: &str) {
        let s = s.trim();
        if !s.is_empty() {
            let v: Vec<char> = s.to_lowercase().chars().collect();
            let v_len = v.len();
            self.add_from_vec_chars(&v, v_len, 0);
        }
    }

    pub fn add_from_vec_chars(&self, v: &[char], v_len: usize, char_index: usize) {
        if v_len > 0 {
            MinStructLetterTrie::add_from_vec_chars_rc(&self.node, v, v_len, char_index);
        }
    }

    fn add_from_vec_chars_rc(rc: &RcRefNode, v: &[char], v_len: usize, char_index: usize) {
        if char_index < v_len {
            let c = v[char_index];
            let mut node = rc.borrow_mut();
            let child_node_opt = node.children.get(&c);

            if USE_CHAR_GET_COUNTER {
                CharGetCounter::record(child_node_opt.is_some());
            }

            if let Some(child_node_rc) = child_node_opt {
                MinStructLetterTrie::add_from_vec_chars_rc(&child_node_rc, v, v_len, char_index + 1);
            } else {
                let new_child_rc: RcRefNode = MinStructLetterTrie::make_node_rc(c);
                MinStructLetterTrie::add_from_vec_chars_rc(&new_child_rc, v, v_len, char_index + 1);
                node.children.insert(c, new_child_rc);
            }
        }
    }

    pub fn merge(&self, other: MinStructLetterTrie) {
        let mut this_node = self.node.borrow_mut();
        for other_child_node_rc in other.node.borrow().children.values() {
            // other_child_node_rc is an RcRefNode.
            let other_child_node = other_child_node_rc.borrow_mut();
            let c = other_child_node.c;
            this_node.children.insert(c, Rc::clone(other_child_node_rc));
        }
    }

    fn print(&self, detail_level: usize) {
        match detail_level {
            1 => println!("{:?}", self.to_fixed_node()),
            2 => println!("{:#?}", self.to_fixed_node()),
            _ => (),
        }
    }

    fn load_read_vec_fill(&self, filename: &str, opt: &DisplayDetailOptions) {
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

    fn load_vec_fill(&self, filename: &str, opt: &DisplayDetailOptions) {
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

    fn load_continuous(&self, filename: &str) {
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

    fn load_continuous_parallel(&self, filename: &str) {
        let (tx, rx) = mpsc::channel();

        let file = File::open(filename).unwrap();

        let mut thread_count = 0;
        let mut prev_c = ' ';
        let mut this_vec: Vec<Vec<char>> = vec![];
        for line in BufReader::new(file).lines() {
            let line = line.unwrap();
            let line = line.trim();
            if !line.is_empty() {
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

        for (received_index, received) in rx.iter().enumerate() {
            self.merge(received);
            if received_index == thread_count - 1 {
                break;
            }
        }
    }

    // Returns the number of threads spawned, which will be 1 if there are items in the vector, otherwise 0.
    fn create_thread_for_part_of_vec(
        v: Vec<Vec<char>>,
        tx: mpsc::Sender<MinStructLetterTrie>,
    ) -> usize {
        if !v.is_empty() {
            thread::spawn(move || {
                let t = MinStructLetterTrie::new();
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
}

impl LetterTrie for MinStructLetterTrie {
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
        let t = Self::new();
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
        self.node.borrow().find_child(prefix, prefix_len, 0)
    }

    fn to_fixed_node(&self) -> FixedNode {
        self.node.borrow().to_fixed_node()
    }
}

impl Debug for MinStructLetterTrie {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.node.borrow().fmt(f)
    }
}

unsafe impl Send for MinStructLetterTrie {}

pub struct MinStructLetterTrieIteratorBreadthFirst {
    stack: Vec<RcRefNode>,
}

impl Iterator for MinStructLetterTrieIteratorBreadthFirst {
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

struct Node {
    c: char,
    children: BTreeMap<char, RcRefNode>,
}

impl Node {
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

    fn to_fixed_node(&self) -> FixedNode {
        FixedNode {
            c: self.c,
            prefix: String::from(""),
            depth: 0,
            is_word: false,
            child_count: 0,
            node_count: 0,
            word_count: 0,
            height: 0,
        }
    }

    pub fn describe_one_line(&self) -> String {
        format!("Node: {}", self.c)
    }

    pub fn describe_deep(&self, s: &mut String, depth: usize) {
        s.push_str(&format!(
            "{}\n",
            format_indent(depth, &(self.describe_one_line()))
        ));
        if depth < DEBUG_TREE_MAX_DEPTH {
            for child_node in self
                .children
                .values()
                .map(|x| x.borrow())
                .take(DEBUG_TREE_MAX_CHILDREN)
            {
                child_node.describe_deep(s, depth + 1);
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

