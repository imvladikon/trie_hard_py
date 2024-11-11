// Cargo.toml additions
// [dependencies]
// pyo3 = { version = "0.16", features = ["extension-module"] }

use pyo3::prelude::*;

use std::collections::BTreeMap;
use std::sync::Arc;

#[pyclass]
#[derive(Debug, Clone)]
pub struct PyTrie {
    trie: TrieHard,
}

#[pymethods]
impl PyTrie {
    #[new]
    pub fn new(items: Vec<String>) -> Self {
        let trie = TrieHard::new(items);
        PyTrie { trie }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.trie.get(key).cloned()
    }

    pub fn contains(&self, key: &str) -> bool {
        self.trie.get(key).is_some()
    }

    pub fn iter(&self) -> PyTrieIterator {
        let items = self.trie.collect_items();
        PyTrieIterator {
            items,
            index: 0,
        }
    }

    pub fn prefix_search(&self, prefix: &str) -> PyTrieIterator {
        let items = self.trie.collect_prefix_items(prefix);
        PyTrieIterator {
            items,
            index: 0,
        }
    }

    pub fn insert(&mut self, key: &str, value: &str) {
        self.trie.insert(key, value.to_string());
    }

    pub fn remove(&mut self, key: &str) {
        self.trie.insert(key, String::new());
    }

    pub fn clear(&mut self) {
        self.trie = TrieHard {
            root: TrieNode::Branch {
                children: BTreeMap::new(),
                value: None,
            },
        };
    }

    pub fn len(&self) -> usize {
        self.trie.collect_items().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn keys(&self) -> Vec<String> {
        self.trie.collect_items().iter().map(|(k, _)| String::from_utf8((**k).clone()).unwrap_or_default()).collect()
    }

    pub fn values(&self) -> Vec<String> {
        self.trie.collect_items().iter().map(|(_, v)| v.clone()).collect()
    }

    pub fn items(&self) -> Vec<(String, String)> {
        self.trie.collect_items().iter().map(|(k, v)| (String::from_utf8((**k).clone()).unwrap_or_default(), v.clone())).collect()
    }

    pub fn starts_with(&self, prefix: &str) -> bool {
        !self.trie.collect_prefix_items(prefix).is_empty()
    }

    pub fn prefix_contains(&self, prefix: &str) -> bool {
        self.trie.collect_prefix_items(prefix).iter().any(|(_, v)| !v.is_empty())
    }
}

#[pyclass]
pub struct PyTrieIterator {
    items: Vec<(Arc<Vec<u8>>, String)>,
    index: usize,
}

#[pymethods]
impl PyTrieIterator {
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    fn __next__(&mut self) -> Option<(String, String)> {
        if self.index < self.items.len() {
            let (k, v) = &self.items[self.index];
            self.index += 1;
            Some((
                String::from_utf8((**k).clone()).unwrap_or_default(),
                v.clone(),
            ))
        } else {
            None
        }
    }
}

#[pymodule]
fn trie_hard_py(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyTrie>()?;
    m.add_class::<PyTrieIterator>()?;
    Ok(())
}

// Simplified Rust implementation of the Trie
#[derive(Debug, Clone)]
enum TrieNode {
    Branch {
        children: BTreeMap<u8, TrieNode>,
        value: Option<String>,
    },
}

#[derive(Debug, Clone)]
pub struct TrieHard {
    root: TrieNode,
}

impl TrieHard {
    pub fn new(items: Vec<String>) -> Self {
        let mut trie = TrieHard {
            root: TrieNode::Branch {
                children: BTreeMap::new(),
                value: None,
            },
        };
        for item in items {
            trie.insert(&item, item.clone());
        }
        trie
    }

    pub fn insert(&mut self, key: &str, value: String) {
        let bytes = key.as_bytes();
        let mut node = &mut self.root;
        for byte in bytes {
            node = match node {
                TrieNode::Branch { children, .. } => {
                    children.entry(*byte).or_insert_with(|| TrieNode::Branch {
                        children: BTreeMap::new(),
                        value: None,
                    })
                }
            };
        }
        if let TrieNode::Branch { value: v, .. } = node {
            *v = Some(value);
        }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        let bytes = key.as_bytes();
        let mut node = &self.root;
        for byte in bytes {
            node = match node {
                TrieNode::Branch { children, .. } => children.get(byte)?,
            };
        }
        if let TrieNode::Branch { value, .. } = node {
            value.as_ref()
        } else {
            None
        }
    }

    pub fn collect_items(&self) -> Vec<(Arc<Vec<u8>>, String)> {
        let mut items = Vec::new();
        self.collect(&self.root, Arc::new(vec![]), &mut items);
        items
    }

    fn collect(&self, node: &TrieNode, prefix: Arc<Vec<u8>>, items: &mut Vec<(Arc<Vec<u8>>, String)>) {
        match node {
            TrieNode::Branch { children, value } => {
                if let Some(v) = value {
                    items.push((prefix.clone(), v.clone()));
                }
                for (byte, child) in children {
                    let mut new_prefix = (*prefix).clone();
                    new_prefix.push(*byte);
                    self.collect(child, Arc::new(new_prefix), items);
                }
            }
        }
    }

    pub fn collect_prefix_items(&self, prefix_str: &str) -> Vec<(Arc<Vec<u8>>, String)> {
        let bytes = prefix_str.as_bytes();
        let mut node = &self.root;
        for byte in bytes {
            node = match node {
                TrieNode::Branch { children, .. } => {
                    if let Some(child) = children.get(byte) {
                        child
                    } else {
                        return Vec::new();
                    }
                }
            };
        }
        let mut items = Vec::new();
        self.collect(node, Arc::new(bytes.to_vec()), &mut items);
        items
    }
}
