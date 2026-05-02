use pyo3::prelude::*;
use std::sync::Arc;

type StoredValue = Arc<str>;

#[pyclass(name = "PyTrie", skip_from_py_object)]
#[derive(Debug, Clone, Default)]
pub struct PyTrie {
    trie: Trie,
}

#[pymethods]
impl PyTrie {
    #[new]
    #[pyo3(signature = (items=None))]
    pub fn new(items: Option<Vec<String>>) -> Self {
        let mut trie = Trie::default();
        if let Some(items) = items {
            trie.extend_keys(items);
        }
        Self { trie }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.trie.get(key).map(ToOwned::to_owned)
    }

    #[pyo3(signature = (key, default=None))]
    pub fn get_or(&self, key: &str, default: Option<String>) -> Option<String> {
        self.get(key).or(default)
    }

    pub fn contains(&self, key: &str) -> bool {
        self.trie.contains_key(key)
    }

    pub fn starts_with(&self, prefix: &str) -> bool {
        self.trie.has_prefix(prefix)
    }

    pub fn prefix_contains(&self, prefix: &str) -> bool {
        self.trie.contains_key(prefix)
    }

    pub fn insert(&mut self, key: &str, value: &str) -> Option<String> {
        self.trie.insert(key, value).map(arc_to_string)
    }

    pub fn add(&mut self, key: &str) -> Option<String> {
        self.trie.insert(key, key).map(arc_to_string)
    }

    pub fn update(&mut self, items: Vec<(String, String)>) {
        for (key, value) in items {
            self.trie.insert(&key, value);
        }
    }

    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.trie.remove(key).map(arc_to_string)
    }

    pub fn discard(&mut self, key: &str) -> bool {
        self.remove(key).is_some()
    }

    pub fn clear(&mut self) {
        self.trie.clear();
    }

    pub fn len(&self) -> usize {
        self.trie.len()
    }

    pub fn is_empty(&self) -> bool {
        self.trie.is_empty()
    }

    pub fn keys(&self) -> Vec<String> {
        self.trie.keys()
    }

    pub fn values(&self) -> Vec<String> {
        self.trie.values()
    }

    pub fn items(&self) -> Vec<(String, String)> {
        self.trie.items()
    }

    pub fn iter(&self) -> PyTrieIterator {
        PyTrieIterator::new(self.items())
    }

    pub fn prefix_search(&self, prefix: &str) -> PyTrieIterator {
        PyTrieIterator::new(self.trie.prefix_items(prefix))
    }

    pub fn longest_prefix(&self, query: &str) -> Option<(String, String)> {
        self.trie.longest_prefix(query)
    }

    pub fn freeze(&self) -> PyFrozenTrie {
        PyFrozenTrie {
            trie: FrozenTrie::from_trie(&self.trie),
        }
    }

    #[pyo3(signature = (query, max_distance=2, limit=10))]
    pub fn fuzzy_search(
        &self,
        query: &str,
        max_distance: usize,
        limit: usize,
    ) -> Vec<(String, String, usize)> {
        self.trie.fuzzy_search(query, max_distance, limit)
    }

    #[pyo3(signature = (query, max_distance=2))]
    pub fn fuzzy_match(&self, query: &str, max_distance: usize) -> Option<(String, String, usize)> {
        self.trie
            .fuzzy_search(query, max_distance, 1)
            .into_iter()
            .next()
    }

    fn __len__(&self) -> usize {
        self.len()
    }

    fn __bool__(&self) -> bool {
        !self.is_empty()
    }

    fn __contains__(&self, key: &str) -> bool {
        self.contains(key)
    }

    fn __getitem__(&self, key: &str) -> PyResult<String> {
        self.get(key)
            .ok_or_else(|| pyo3::exceptions::PyKeyError::new_err(key.to_owned()))
    }

    fn __setitem__(&mut self, key: &str, value: &str) {
        self.insert(key, value);
    }

    fn __delitem__(&mut self, key: &str) -> PyResult<()> {
        self.remove(key)
            .map(|_| ())
            .ok_or_else(|| pyo3::exceptions::PyKeyError::new_err(key.to_owned()))
    }

    fn __iter__(&self) -> PyTrieIterator {
        PyTrieIterator::new(self.items())
    }

    fn __repr__(&self) -> String {
        format!("PyTrie(len={})", self.len())
    }
}

#[pyclass(name = "PyFrozenTrie", skip_from_py_object)]
#[derive(Debug, Clone, Default)]
pub struct PyFrozenTrie {
    trie: FrozenTrie,
}

#[pymethods]
impl PyFrozenTrie {
    pub fn get(&self, key: &str) -> Option<String> {
        self.trie.get(key).map(ToOwned::to_owned)
    }

    #[pyo3(signature = (key, default=None))]
    pub fn get_or(&self, key: &str, default: Option<String>) -> Option<String> {
        self.get(key).or(default)
    }

    pub fn contains(&self, key: &str) -> bool {
        self.trie.contains_key(key)
    }

    pub fn starts_with(&self, prefix: &str) -> bool {
        self.trie.has_prefix(prefix)
    }

    pub fn prefix_contains(&self, prefix: &str) -> bool {
        self.trie.contains_key(prefix)
    }

    pub fn len(&self) -> usize {
        self.trie.len()
    }

    pub fn is_empty(&self) -> bool {
        self.trie.is_empty()
    }

    pub fn keys(&self) -> Vec<String> {
        self.trie.keys()
    }

    pub fn values(&self) -> Vec<String> {
        self.trie.values()
    }

    pub fn items(&self) -> Vec<(String, String)> {
        self.trie.items()
    }

    pub fn iter(&self) -> PyTrieIterator {
        PyTrieIterator::new(self.items())
    }

    pub fn prefix_search(&self, prefix: &str) -> PyTrieIterator {
        PyTrieIterator::new(self.trie.prefix_items(prefix))
    }

    pub fn longest_prefix(&self, query: &str) -> Option<(String, String)> {
        self.trie.longest_prefix(query)
    }

    #[pyo3(signature = (query, max_distance=2, limit=10))]
    pub fn fuzzy_search(
        &self,
        query: &str,
        max_distance: usize,
        limit: usize,
    ) -> Vec<(String, String, usize)> {
        self.trie.fuzzy_search(query, max_distance, limit)
    }

    #[pyo3(signature = (query, max_distance=2))]
    pub fn fuzzy_match(&self, query: &str, max_distance: usize) -> Option<(String, String, usize)> {
        self.trie
            .fuzzy_search(query, max_distance, 1)
            .into_iter()
            .next()
    }

    fn __len__(&self) -> usize {
        self.len()
    }

    fn __bool__(&self) -> bool {
        !self.is_empty()
    }

    fn __contains__(&self, key: &str) -> bool {
        self.contains(key)
    }

    fn __getitem__(&self, key: &str) -> PyResult<String> {
        self.get(key)
            .ok_or_else(|| pyo3::exceptions::PyKeyError::new_err(key.to_owned()))
    }

    fn __iter__(&self) -> PyTrieIterator {
        PyTrieIterator::new(self.items())
    }

    fn __repr__(&self) -> String {
        format!("PyFrozenTrie(len={})", self.len())
    }
}

#[pyclass(skip_from_py_object)]
#[derive(Debug, Clone)]
pub struct PyTrieIterator {
    items: Vec<(String, String)>,
    index: usize,
}

impl PyTrieIterator {
    fn new(items: Vec<(String, String)>) -> Self {
        Self { items, index: 0 }
    }
}

#[pymethods]
impl PyTrieIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(&mut self) -> Option<(String, String)> {
        let item = self.items.get(self.index).cloned();
        self.index += usize::from(item.is_some());
        item
    }
}

#[pymodule]
fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyTrie>()?;
    m.add_class::<PyFrozenTrie>()?;
    m.add_class::<PyTrieIterator>()?;
    Ok(())
}

#[derive(Debug, Clone)]
struct Trie {
    nodes: Vec<TrieNode>,
    len: usize,
}

impl Default for Trie {
    fn default() -> Self {
        Self {
            nodes: vec![TrieNode::default()],
            len: 0,
        }
    }
}

impl Trie {
    fn extend_keys(&mut self, keys: impl IntoIterator<Item = String>) {
        for key in keys {
            self.insert(&key.clone(), key);
        }
    }

    fn insert(&mut self, key: &str, value: impl Into<StoredValue>) -> Option<StoredValue> {
        let mut node_index = 0;
        for byte in key.bytes() {
            node_index = match self.child_position(node_index, byte) {
                Ok(position) => self.nodes[node_index].children[position].1,
                Err(position) => {
                    let new_node_index = self.nodes.len();
                    self.nodes.push(TrieNode::default());
                    self.nodes[node_index]
                        .children
                        .insert(position, (byte, new_node_index));
                    new_node_index
                }
            };
        }

        let previous = self.nodes[node_index].value.replace(value.into());
        if previous.is_none() {
            self.len += 1;
        }
        previous
    }

    fn get(&self, key: &str) -> Option<&str> {
        self.find_node_index(key)
            .and_then(|node_index| self.nodes[node_index].value.as_deref())
    }

    fn contains_key(&self, key: &str) -> bool {
        self.get(key).is_some()
    }

    fn has_prefix(&self, prefix: &str) -> bool {
        self.find_node_index(prefix).is_some()
    }

    fn remove(&mut self, key: &str) -> Option<StoredValue> {
        let (removed, _) = self.remove_inner(0, key.as_bytes(), 0);
        if removed.is_some() {
            self.len -= 1;
        }
        removed
    }

    fn remove_inner(
        &mut self,
        node_index: usize,
        key: &[u8],
        depth: usize,
    ) -> (Option<StoredValue>, bool) {
        if depth == key.len() {
            let node = &mut self.nodes[node_index];
            let removed = node.value.take();
            let should_prune = removed.is_some() && node.children.is_empty();
            return (removed, should_prune);
        }

        let Ok(child_position) = self.child_position(node_index, key[depth]) else {
            return (None, false);
        };
        let child_index = self.nodes[node_index].children[child_position].1;

        let (removed, child_should_prune) = self.remove_inner(child_index, key, depth + 1);
        if child_should_prune {
            self.nodes[node_index].children.remove(child_position);
        }

        let node = &self.nodes[node_index];
        let should_prune = removed.is_some() && node.value.is_none() && node.children.is_empty();
        (removed, should_prune)
    }

    fn clear(&mut self) {
        self.nodes.clear();
        self.nodes.push(TrieNode::default());
        self.len = 0;
    }

    fn len(&self) -> usize {
        self.len
    }

    fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn keys(&self) -> Vec<String> {
        self.items().into_iter().map(|(key, _)| key).collect()
    }

    fn values(&self) -> Vec<String> {
        self.items().into_iter().map(|(_, value)| value).collect()
    }

    fn items(&self) -> Vec<(String, String)> {
        let mut items = Vec::with_capacity(self.len);
        self.collect(0, &mut Vec::new(), &mut items);
        items
    }

    fn prefix_items(&self, prefix: &str) -> Vec<(String, String)> {
        let Some(node_index) = self.find_node_index(prefix) else {
            return Vec::new();
        };

        let mut items = Vec::new();
        self.collect(node_index, &mut prefix.as_bytes().to_vec(), &mut items);
        items
    }

    fn longest_prefix(&self, query: &str) -> Option<(String, String)> {
        let mut node_index = 0;
        let mut best = self.nodes[node_index]
            .value
            .as_ref()
            .map(|value| ("".to_owned(), value.to_string()));
        let mut prefix = Vec::new();

        for byte in query.bytes() {
            let Some(next_index) = self.child_node_index(node_index, byte) else {
                break;
            };
            prefix.push(byte);
            node_index = next_index;

            if let Some(value) = &self.nodes[node_index].value {
                best = Some((bytes_to_string(&prefix), value.to_string()));
            }
        }

        best
    }

    fn fuzzy_search(
        &self,
        query: &str,
        max_distance: usize,
        limit: usize,
    ) -> Vec<(String, String, usize)> {
        if limit == 0 {
            return Vec::new();
        }

        let query_chars = query.chars().collect::<Vec<_>>();
        let mut search = FuzzySearch {
            query: &query_chars,
            max_distance,
            matches: Vec::new(),
        };
        let initial_row = (0..=query_chars.len()).collect::<Vec<_>>();

        self.collect_fuzzy(
            0,
            &initial_row,
            &mut Vec::new(),
            &mut Vec::new(),
            &mut search,
        );

        let mut matches = search.matches;
        matches.sort_by(|left, right| left.2.cmp(&right.2).then_with(|| left.0.cmp(&right.0)));
        matches.truncate(limit);
        matches
    }

    fn find_node_index(&self, key: &str) -> Option<usize> {
        let mut node_index = 0;
        for byte in key.bytes() {
            node_index = self.child_node_index(node_index, byte)?;
        }
        Some(node_index)
    }

    fn collect(&self, node_index: usize, prefix: &mut Vec<u8>, items: &mut Vec<(String, String)>) {
        let node = &self.nodes[node_index];
        if let Some(value) = &node.value {
            items.push((bytes_to_string(prefix), value.to_string()));
        }

        for (byte, child_index) in &node.children {
            prefix.push(*byte);
            self.collect(*child_index, prefix, items);
            prefix.pop();
        }
    }

    fn collect_fuzzy(
        &self,
        node_index: usize,
        previous_row: &[usize],
        prefix: &mut Vec<u8>,
        pending_char: &mut Vec<u8>,
        search: &mut FuzzySearch<'_>,
    ) {
        let node = &self.nodes[node_index];
        if pending_char.is_empty() {
            if let Some(value) = &node.value {
                let distance = previous_row[search.query.len()];
                if distance <= search.max_distance {
                    search
                        .matches
                        .push((bytes_to_string(prefix), value.to_string(), distance));
                }
            }
        }

        for (byte, child) in &node.children {
            let saved_pending = pending_char.clone();
            prefix.push(*byte);
            pending_char.push(*byte);

            match std::str::from_utf8(pending_char) {
                Ok(char_str) => {
                    if let Some(character) = single_char(char_str) {
                        let current_row = levenshtein_row(character, search.query, previous_row);
                        if current_row
                            .iter()
                            .min()
                            .is_some_and(|distance| *distance <= search.max_distance)
                        {
                            pending_char.clear();
                            self.collect_fuzzy(*child, &current_row, prefix, pending_char, search);
                        }
                    }
                }
                Err(error) if error.error_len().is_none() => {
                    self.collect_fuzzy(*child, previous_row, prefix, pending_char, search);
                }
                Err(_) => {}
            }

            *pending_char = saved_pending;
            prefix.pop();
        }
    }

    fn child_position(&self, node_index: usize, byte: u8) -> Result<usize, usize> {
        self.nodes[node_index]
            .children
            .binary_search_by_key(&byte, |(child_byte, _)| *child_byte)
    }

    fn child_node_index(&self, node_index: usize, byte: u8) -> Option<usize> {
        self.child_position(node_index, byte)
            .ok()
            .map(|position| self.nodes[node_index].children[position].1)
    }
}

#[derive(Debug, Clone, Default)]
struct TrieNode {
    children: Vec<(u8, usize)>,
    value: Option<StoredValue>,
}

#[derive(Debug, Clone, Default)]
struct FrozenTrie {
    nodes: Vec<FrozenNode>,
    child_masks: Vec<ByteMask>,
    len: usize,
}

impl FrozenTrie {
    const MASK_CHILD_THRESHOLD: usize = 8;

    fn from_trie(trie: &Trie) -> Self {
        let mut frozen = Self {
            nodes: vec![FrozenNode::default()],
            child_masks: Vec::new(),
            len: trie.len(),
        };
        let mut queue = vec![(0, 0)];
        let mut cursor = 0;

        while cursor < queue.len() {
            let (source_index, frozen_index) = queue[cursor];
            cursor += 1;

            let source = &trie.nodes[source_index];
            let first_child = frozen.nodes.len();
            let child_count = source.children.len();

            frozen.nodes[frozen_index].value.clone_from(&source.value);
            frozen.nodes[frozen_index].first_child = first_child;
            frozen.nodes[frozen_index].child_count = child_count;

            let mut child_mask =
                (child_count >= Self::MASK_CHILD_THRESHOLD).then(ByteMask::default);
            for (byte, source_child_index) in &source.children {
                if let Some(mask) = &mut child_mask {
                    mask.insert(*byte);
                }
                let child_frozen_index = frozen.nodes.len();
                frozen.nodes.push(FrozenNode {
                    incoming_byte: *byte,
                    ..FrozenNode::default()
                });
                queue.push((*source_child_index, child_frozen_index));
            }

            if let Some(mask) = child_mask {
                frozen.nodes[frozen_index].child_mask_index = Some(frozen.child_masks.len());
                frozen.child_masks.push(mask);
            }
        }

        frozen
    }

    fn get(&self, key: &str) -> Option<&str> {
        self.find_node_index(key)
            .and_then(|node_index| self.nodes[node_index].value.as_deref())
    }

    fn contains_key(&self, key: &str) -> bool {
        self.get(key).is_some()
    }

    fn has_prefix(&self, prefix: &str) -> bool {
        self.find_node_index(prefix).is_some()
    }

    fn len(&self) -> usize {
        self.len
    }

    fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn keys(&self) -> Vec<String> {
        self.items().into_iter().map(|(key, _)| key).collect()
    }

    fn values(&self) -> Vec<String> {
        self.items().into_iter().map(|(_, value)| value).collect()
    }

    fn items(&self) -> Vec<(String, String)> {
        let mut items = Vec::with_capacity(self.len);
        self.collect(0, &mut Vec::new(), &mut items);
        items
    }

    fn prefix_items(&self, prefix: &str) -> Vec<(String, String)> {
        let Some(node_index) = self.find_node_index(prefix) else {
            return Vec::new();
        };

        let mut items = Vec::new();
        self.collect(node_index, &mut prefix.as_bytes().to_vec(), &mut items);
        items
    }

    fn longest_prefix(&self, query: &str) -> Option<(String, String)> {
        let mut node_index = 0;
        let mut best = self.nodes[node_index]
            .value
            .as_ref()
            .map(|value| ("".to_owned(), value.to_string()));
        let mut prefix = Vec::new();

        for byte in query.bytes() {
            let Some(next_index) = self.child_node_index(node_index, byte) else {
                break;
            };
            prefix.push(byte);
            node_index = next_index;

            if let Some(value) = &self.nodes[node_index].value {
                best = Some((bytes_to_string(&prefix), value.to_string()));
            }
        }

        best
    }

    fn fuzzy_search(
        &self,
        query: &str,
        max_distance: usize,
        limit: usize,
    ) -> Vec<(String, String, usize)> {
        if limit == 0 {
            return Vec::new();
        }

        let query_chars = query.chars().collect::<Vec<_>>();
        let mut search = FuzzySearch {
            query: &query_chars,
            max_distance,
            matches: Vec::new(),
        };
        let initial_row = (0..=query_chars.len()).collect::<Vec<_>>();

        self.collect_fuzzy(
            0,
            &initial_row,
            &mut Vec::new(),
            &mut Vec::new(),
            &mut search,
        );

        let mut matches = search.matches;
        matches.sort_by(|left, right| left.2.cmp(&right.2).then_with(|| left.0.cmp(&right.0)));
        matches.truncate(limit);
        matches
    }

    fn find_node_index(&self, key: &str) -> Option<usize> {
        let mut node_index = 0;
        for byte in key.bytes() {
            node_index = self.child_node_index(node_index, byte)?;
        }
        Some(node_index)
    }

    fn collect(&self, node_index: usize, prefix: &mut Vec<u8>, items: &mut Vec<(String, String)>) {
        let node = &self.nodes[node_index];
        if let Some(value) = &node.value {
            items.push((bytes_to_string(prefix), value.to_string()));
        }

        for child_offset in 0..node.child_count {
            let child_index = node.first_child + child_offset;
            let byte = self.nodes[child_index].incoming_byte;
            prefix.push(byte);
            self.collect(child_index, prefix, items);
            prefix.pop();
        }
    }

    fn collect_fuzzy(
        &self,
        node_index: usize,
        previous_row: &[usize],
        prefix: &mut Vec<u8>,
        pending_char: &mut Vec<u8>,
        search: &mut FuzzySearch<'_>,
    ) {
        let node = &self.nodes[node_index];
        if pending_char.is_empty() {
            if let Some(value) = &node.value {
                let distance = previous_row[search.query.len()];
                if distance <= search.max_distance {
                    search
                        .matches
                        .push((bytes_to_string(prefix), value.to_string(), distance));
                }
            }
        }

        for child_offset in 0..node.child_count {
            let child_index = node.first_child + child_offset;
            let byte = self.nodes[child_index].incoming_byte;
            let saved_pending = pending_char.clone();
            prefix.push(byte);
            pending_char.push(byte);

            match std::str::from_utf8(pending_char) {
                Ok(char_str) => {
                    if let Some(character) = single_char(char_str) {
                        let current_row = levenshtein_row(character, search.query, previous_row);
                        if current_row
                            .iter()
                            .min()
                            .is_some_and(|distance| *distance <= search.max_distance)
                        {
                            pending_char.clear();
                            self.collect_fuzzy(
                                child_index,
                                &current_row,
                                prefix,
                                pending_char,
                                search,
                            );
                        }
                    }
                }
                Err(error) if error.error_len().is_none() => {
                    self.collect_fuzzy(child_index, previous_row, prefix, pending_char, search);
                }
                Err(_) => {}
            }

            *pending_char = saved_pending;
            prefix.pop();
        }
    }

    fn child_node_index(&self, node_index: usize, byte: u8) -> Option<usize> {
        let node = &self.nodes[node_index];
        if let Some(mask_index) = node.child_mask_index {
            let rank = self.child_masks[mask_index].rank(byte)?;
            return (rank < node.child_count).then_some(node.first_child + rank);
        }

        let child_slice = &self.nodes[node.first_child..node.first_child + node.child_count];
        let rank = child_slice
            .binary_search_by_key(&byte, |child| child.incoming_byte)
            .ok()?;
        Some(node.first_child + rank)
    }
}

#[derive(Debug, Clone, Default)]
struct FrozenNode {
    incoming_byte: u8,
    value: Option<StoredValue>,
    child_mask_index: Option<usize>,
    first_child: usize,
    child_count: usize,
}

#[derive(Debug, Clone, Copy, Default)]
struct ByteMask([u64; 4]);

impl ByteMask {
    fn insert(&mut self, byte: u8) {
        self.0[usize::from(byte / 64)] |= 1_u64 << u32::from(byte % 64);
    }

    fn rank(&self, byte: u8) -> Option<usize> {
        let bucket = usize::from(byte / 64);
        let bit = u32::from(byte % 64);
        let mask = 1_u64 << bit;

        if self.0[bucket] & mask == 0 {
            return None;
        }

        let lower_in_bucket = self.0[bucket] & (mask - 1);
        let lower_buckets = self.0[..bucket]
            .iter()
            .map(|bits| bits.count_ones() as usize)
            .sum::<usize>();

        Some(lower_buckets + lower_in_bucket.count_ones() as usize)
    }
}

struct FuzzySearch<'a> {
    query: &'a [char],
    max_distance: usize,
    matches: Vec<(String, String, usize)>,
}

fn bytes_to_string(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).expect("keys are inserted from valid UTF-8 strings")
}

fn arc_to_string(value: StoredValue) -> String {
    value.to_string()
}

#[cfg(test)]
fn bounded_levenshtein(candidate: &str, query: &[char], max_distance: usize) -> Option<usize> {
    let candidate_len = candidate.chars().count();
    let query_len = query.len();

    if candidate_len.abs_diff(query_len) > max_distance {
        return None;
    }

    let mut previous = (0..=query_len).collect::<Vec<_>>();
    let mut current = vec![0; query_len + 1];

    for (row, candidate_char) in candidate.chars().enumerate() {
        current[0] = row + 1;
        let mut row_min = current[0];

        for (col, query_char) in query.iter().enumerate() {
            let substitution_cost = usize::from(candidate_char != *query_char);
            let insertion = current[col] + 1;
            let deletion = previous[col + 1] + 1;
            let substitution = previous[col] + substitution_cost;
            let distance = insertion.min(deletion).min(substitution);

            current[col + 1] = distance;
            row_min = row_min.min(distance);
        }

        if row_min > max_distance {
            return None;
        }

        std::mem::swap(&mut previous, &mut current);
    }

    (previous[query_len] <= max_distance).then_some(previous[query_len])
}

fn levenshtein_row(character: char, query: &[char], previous_row: &[usize]) -> Vec<usize> {
    let mut current_row = vec![previous_row[0] + 1; query.len() + 1];

    for (col, query_char) in query.iter().enumerate() {
        let substitution_cost = usize::from(character != *query_char);
        let insertion = current_row[col] + 1;
        let deletion = previous_row[col + 1] + 1;
        let substitution = previous_row[col] + substitution_cost;

        current_row[col + 1] = insertion.min(deletion).min(substitution);
    }

    current_row
}

fn single_char(value: &str) -> Option<char> {
    let mut chars = value.chars();
    let character = chars.next()?;
    chars.next().is_none().then_some(character)
}

#[cfg(test)]
mod tests {
    use super::{bounded_levenshtein, FrozenTrie, Trie};

    #[test]
    fn insert_get_and_order_items() {
        let mut trie = Trie::default();
        trie.insert("dad", "DAD".to_owned());
        trie.insert("ant", "ANT".to_owned());
        trie.insert("and", "AND".to_owned());

        assert_eq!(trie.get("ant"), Some("ANT"));
        assert_eq!(trie.get("an"), None);
        assert_eq!(
            trie.items(),
            vec![
                ("and".to_owned(), "AND".to_owned()),
                ("ant".to_owned(), "ANT".to_owned()),
                ("dad".to_owned(), "DAD".to_owned()),
            ]
        );
    }

    #[test]
    fn remove_deletes_only_requested_key_and_prunes_dead_branches() {
        let mut trie = Trie::default();
        trie.insert("a", "a".to_owned());
        trie.insert("aa", "aa".to_owned());
        trie.insert("ab", "ab".to_owned());

        assert_eq!(trie.remove("aa").as_deref(), Some("aa"));
        assert_eq!(trie.remove("missing"), None);
        assert_eq!(trie.get("a"), Some("a"));
        assert_eq!(trie.get("ab"), Some("ab"));
        assert_eq!(trie.get("aa"), None);
        assert_eq!(trie.len(), 2);
    }

    #[test]
    fn prefix_search_and_longest_prefix_are_byte_safe() {
        let mut trie = Trie::default();
        trie.insert("bär", "short".to_owned());
        trie.insert("bären", "long".to_owned());
        trie.insert("bear", "ascii".to_owned());

        assert_eq!(
            trie.prefix_items("bä"),
            vec![
                ("bär".to_owned(), "short".to_owned()),
                ("bären".to_owned(), "long".to_owned()),
            ]
        );
        assert_eq!(
            trie.longest_prefix("bärenstark"),
            Some(("bären".to_owned(), "long".to_owned()))
        );
    }

    #[test]
    fn fuzzy_search_ranks_matches_by_distance_then_key() {
        let mut trie = Trie::default();
        trie.insert("cart", "cart".to_owned());
        trie.insert("cat", "cat".to_owned());
        trie.insert("cot", "cot".to_owned());
        trie.insert("dog", "dog".to_owned());

        assert_eq!(
            trie.fuzzy_search("cut", 1, 10),
            vec![
                ("cat".to_owned(), "cat".to_owned(), 1),
                ("cot".to_owned(), "cot".to_owned(), 1),
            ]
        );
        assert_eq!(trie.fuzzy_search("cut", 1, 1).len(), 1);
    }

    #[test]
    fn levenshtein_counts_unicode_scalars_not_utf8_bytes() {
        let query = "мед".chars().collect::<Vec<_>>();

        assert_eq!(bounded_levenshtein("мёд", &query, 1), Some(1));
        assert_eq!(bounded_levenshtein("медведь", &query, 1), None);
    }

    #[test]
    fn frozen_trie_matches_mutable_queries() {
        let mut trie = Trie::default();
        for key in ["dad", "ant", "and", "dot", "do", "мёд"] {
            trie.insert(key, key.to_owned());
        }

        let frozen = FrozenTrie::from_trie(&trie);
        trie.insert("later", "later".to_owned());

        assert_eq!(frozen.get("do"), Some("do"));
        assert_eq!(frozen.get("later"), None);
        assert_eq!(
            frozen.prefix_items("d"),
            vec![
                ("dad".to_owned(), "dad".to_owned()),
                ("do".to_owned(), "do".to_owned()),
                ("dot".to_owned(), "dot".to_owned()),
            ]
        );
        assert_eq!(
            frozen.longest_prefix("dotnet"),
            Some(("dot".to_owned(), "dot".to_owned()))
        );
        assert_eq!(
            frozen.fuzzy_search("мед", 1, 1),
            vec![("мёд".to_owned(), "мёд".to_owned(), 1)]
        );
    }
}
