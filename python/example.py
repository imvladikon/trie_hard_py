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