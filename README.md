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

trie = PyTrie(["and", "ant", "dad", "do", "dot"])

print(trie.contains("dad"))    # Output: True
print(trie.contains("don't"))  # Output: False

print(trie.get("do"))  # Output: do

for key, value in trie.iter():
    print(f"{key}: {value}")

for key, value in trie.prefix_search("d"):
    print(f"{key}: {value}")

print(trie.prefix_contains("da"))  # Output: True
```