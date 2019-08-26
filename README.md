# letter_trie_1

[![interactive version](https://github.com/David-Thureson/letter_trie_1/blob/master/trie_from_a.png)](https://bl.ocks.org/David-Thureson/raw/f9dd7ba0aab896ae893f31d6636a1870/ "interactive version")

## Overview

A trie (https://www.geeksforgeeks.org/trie-insert-and-search) made from words in a file, with one letter per node.

The image above is an example of a small portion of a trie. To make it viewable the root is at the first letter "a", it shows only 4 levels, and it's built from a list of only the 5,000 most common English words.

My interactive version of a trie from the same 5,000 words is at https://bl.ocks.org/David-Thureson/raw/f9dd7ba0aab896ae893f31d6636a1870/. Click on a node to make that the root. The only way to go back is to refresh the browser window to start again.

One use of such a trie is to play millions of rounds of a word game like Scrabble or Boggle. In either game there's a branching set of possible letter sequences given a set of seven tiles (and the current state of the Scrabble board) or sixteen dice. The trie allows one to work letter-by-letter through the possible sequences while stepping node-by-node through the trie at the same time. It may be possible to cut the process short without either running out of tiles/dice or reaching a leaf node in the trie. For instance, we may be tracking the best score reached so far and a given node could be annotated with the highest score that could possibly be reached in its subtree. If this highest possible score is less than or equal to the best score we've already found, there's no point in going into that subtree or (which is saying the same thing) in trying to add another tile or die to the word we've been forming.

A trie of even half a million words in a given language (with probably fewer than 1.5 million nodes) is still going to be orders of magnitude smaller than the possible sequences of letters found in one throw of the Boggle dice. This means that even if we don't cut the search short because of the best possible score in a subtree, we're still in most cases going to run out of trie before we run out of sequences of dice.

The trie in the demo above and the code in this project couldn't be used as-is for Boggle because that game puts "QU" together on the face of a die as if it's one letter. This makes it far easier for players to form words with "q" but it requires some special handling either when building the trie or when traversing it.

## Important note, or why you should probably leave now

This code is not an attempt to make a practical data structure but instead it's a test bed for learning the [Rust language](https://www.rust-lang.org/). Thus there are two main implementations that use different approaches for links between nodes even though their performance is nearly identical. Also one of the implementations has unnecessary child-to-parent links, simply to experiment with [std::rc::Rc](https://doc.rust-lang.org/std/rc/struct.Rc.html) and [std::cell::RefCell](https://doc.rust-lang.org/beta/std/cell/struct.RefCell.html). Also there are often several functions that do the same thing but using different techniques such as recursion, loops, or iterators. Lastly, it's mostly test and debugging support code surrounding a little bit of data structure code.

So if you've stumbled upon this repository because you were looking for a practical data structure you could use in your Rust projects, you've taken a wrong turn. However, if you were looking for an example of how to learn a language by creating multiple variations on a theme and relentlessly unit testing and benchmarking those variations, then you're in luck.
