## python binding for trie-hard library

simple python binding for the [trie-hard library](https://github.com/cloudflare/trie-hard)

## Build

```bash
maturin build --release 
```

## Install from wheel

```bash
pip install target/wheels/trie_hard_py-0.1.0-cp310-cp310-manylinux_2_34_x86_64.whl
```

## Usage

```python
from trie_hard_py import PyTrie

trie = PyTrie(["PF-102843 150mg", "PF-137492 50mg", "PF-194328 75mg",
               "PF-208364 200mg", "PF-211357 25mg", "PF-220156 100mg",
               "PF-301728 10mg", "Cymbrixal", "Trelvion",
               "Zyntarel", "Protaxil", "Laxiterin",
               "Velarix", "Nebaflo", "Cortyphen",
               "Nexivane", "Dexorin", "Mexitine"])

print(trie.contains("Zyntarel"))  # Output: True
print(trie.contains("Zynt"))  # Output: False

print(trie.get("Cortyphen"))  # Output: do

for key, value in trie.iter():
    print(f"{key}: {value}")

for key, value in trie.prefix_search("PF"):
    print(f"{key}: {value}")

print(trie.prefix_contains("Mexit"))  # Output: True
```