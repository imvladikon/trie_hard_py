import pytest

import trie_hard_py
from trie_hard_py import PyTrie


def test_package_exports_version():
    assert trie_hard_py.__version__ == "0.1.0"


def test_constructor_lookup_and_ordering():
    trie = PyTrie(["dad", "ant", "and", "dot", "do"])

    assert len(trie) == 5
    assert "dad" in trie
    assert "da" not in trie
    assert trie.get("do") == "do"
    assert trie.keys() == ["and", "ant", "dad", "do", "dot"]
    assert trie.items() == [
        ("and", "and"),
        ("ant", "ant"),
        ("dad", "dad"),
        ("do", "do"),
        ("dot", "dot"),
    ]


def test_mapping_methods_and_delete_semantics():
    trie = PyTrie()

    assert not trie
    assert trie.insert("api", "v1") is None
    assert trie.insert("api", "v2") == "v1"
    trie["app"] = "service"

    assert trie["api"] == "v2"
    assert trie.discard("missing") is False
    assert trie.remove("api") == "v2"
    assert "api" not in trie
    assert "app" in trie

    with pytest.raises(KeyError):
        trie["api"]

    del trie["app"]
    assert trie.is_empty()


def test_prefix_search_unicode_and_longest_prefix():
    trie = PyTrie()
    trie.update(
        [
            ("bär", "short"),
            ("bären", "long"),
            ("bear", "ascii"),
            ("beta", "greek"),
        ]
    )

    assert list(trie.prefix_search("bä")) == [("bär", "short"), ("bären", "long")]
    assert trie.starts_with("be") is True
    assert trie.prefix_contains("bär") is True
    assert trie.prefix_contains("bä") is False
    assert trie.longest_prefix("bärenstark") == ("bären", "long")
    assert trie.longest_prefix("unknown") is None


def test_fuzzy_search_and_best_match():
    trie = PyTrie(["cart", "cat", "cot", "dog", "мёд"])

    assert trie.fuzzy_search("cut", max_distance=1, limit=10) == [
        ("cat", "cat", 1),
        ("cot", "cot", 1),
    ]
    assert trie.fuzzy_search("cut", max_distance=1, limit=1) == [("cat", "cat", 1)]
    assert trie.fuzzy_match("мед", max_distance=1) == ("мёд", "мёд", 1)
    assert trie.fuzzy_match("zzzz", max_distance=1) is None


def test_frozen_trie_is_read_only_snapshot_with_same_queries():
    trie = PyTrie(["dad", "ant", "and", "dot", "do", "мёд"])
    frozen = trie.freeze()
    trie.insert("later", "later")

    assert len(frozen) == 6
    assert "later" not in frozen
    assert frozen.get("dad") == "dad"
    assert frozen.keys() == ["and", "ant", "dad", "do", "dot", "мёд"]
    assert list(frozen.prefix_search("d")) == [("dad", "dad"), ("do", "do"), ("dot", "dot")]
    assert frozen.longest_prefix("dotnet") == ("dot", "dot")
    assert frozen.fuzzy_match("мед", max_distance=1) == ("мёд", "мёд", 1)
