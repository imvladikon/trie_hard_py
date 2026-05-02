from trie_hard_py import PyTrie

trie = PyTrie(["dad", "ant", "and", "dot", "do"])
trie.insert("api/v1", "handler_v1")
trie["api/v2"] = "handler_v2"

print(trie.contains("dad"))
print(trie.get("do"))
print(trie.keys())
print(list(trie.prefix_search("api/")))
print(trie.longest_prefix("api/v2/users"))
print(trie.fuzzy_match("api/v3", max_distance=1))
