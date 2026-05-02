from trie_hard_py import PyTrie

trie = PyTrie(["san antonio", "san diego", "san jose", "serbia", "sertraline"])
trie.insert("amoxicillin tablet", "RXNORM:308182")
trie["amoxicillin oral suspension"] = "RXNORM:308189"

print(trie.contains("san diego"))
print(trie.get("amoxicillin tablet"))
print(trie.keys())
print(list(trie.prefix_search("san ")))
print(trie.longest_prefix("amoxicillin tablet 500mg"))
print(trie.fuzzy_match("sertralina", max_distance=1))
