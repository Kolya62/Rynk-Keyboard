#[derive(Default, Clone, Debug)]
pub struct TrieNode {
    pub children: Vec<(char, Box<TrieNode>)>,
    pub frequency: u32,
    pub is_terminal: bool,
    pub canonical_word: Option<String>,
}

impl TrieNode {
    pub fn new() -> Self {
        Self {
            children: Vec::with_capacity(4),
            frequency: 0,
            is_terminal: false,
            canonical_word: None,
        }
    }

    pub fn insert(&mut self, word: &str, frequency: u32) {
        let mut curr = self;
        for ch in word.to_lowercase().chars() {
            let idx = match curr.children.iter().position(|(c, _)| *c == ch) {
                Some(i) => i,
                None => {
                    curr.children.push((ch, Box::new(TrieNode::new())));
                    curr.children.len() - 1
                }
            };
            curr = &mut curr.children[idx].1;
        }
        curr.is_terminal = true;
        if frequency >= curr.frequency || curr.canonical_word.is_none() {
            curr.frequency = curr.frequency.max(frequency);
            curr.canonical_word = Some(word.to_string());
        }
    }

    pub fn find_node(&self, prefix: &str) -> Option<&TrieNode> {
        let mut curr = self;
        for ch in prefix.to_lowercase().chars() {
            match curr.children.iter().find(|(c, _)| *c == ch) {
                Some((_, child)) => curr = child.as_ref(),
                None => return None,
            }
        }
        Some(curr)
    }

    pub fn collect_completions(
        &self,
        current_prefix: &mut String,
        results: &mut Vec<(String, u32)>,
    ) {
        if self.is_terminal {
            let text = self.canonical_word.clone().unwrap_or_else(|| current_prefix.clone());
            results.push((text, self.frequency));
        }

        for (ch, child) in &self.children {
            current_prefix.push(*ch);
            child.collect_completions(current_prefix, results);
            current_prefix.pop();
        }
    }
}

pub struct Trie {
    root: TrieNode,
}

impl Default for Trie {
    fn default() -> Self {
        Self {
            root: TrieNode::new(),
        }
    }
}

impl Trie {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, word: &str, frequency: u32) {
        if !word.is_empty() {
            self.root.insert(word, frequency);
        }
    }

    pub fn find_completions(&self, prefix: &str, limit: usize) -> Vec<(String, u32)> {
        if prefix.is_empty() {
            return Vec::new();
        }

        let clean_prefix = prefix.to_lowercase();
        let mut results = Vec::with_capacity(32);
        if let Some(node) = self.root.find_node(&clean_prefix) {
            let mut buf = clean_prefix;
            node.collect_completions(&mut buf, &mut results);
            results.sort_by_key(|a| std::cmp::Reverse(a.1));
            if results.len() > limit {
                results.truncate(limit);
            }
        }
        results
    }

    pub fn contains(&self, word: &str) -> bool {
        if word.is_empty() {
            return false;
        }
        self.root.find_node(word).map(|n| n.is_terminal).unwrap_or(false)
    }

    pub fn get_frequency(&self, word: &str) -> Option<u32> {
        if word.is_empty() {
            return None;
        }
        self.root.find_node(word).and_then(|n| {
            if n.is_terminal {
                Some(n.frequency)
            } else {
                None
            }
        })
    }
}
