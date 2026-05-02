# trie-hard-py

`trie-hard-py` is a Python package backed by a Rust trie implementation. It is
useful when you need ordered key lookup, prefix search, longest-prefix matching,
and mutable map-style updates from Python.

The public module is `trie_hard_py`; the Rust extension is packaged as
`trie_hard_py._native`.

## Features

- Exact lookup: `get`, `in`, `[]`
- Ordered iteration over `(key, value)` pairs
- Prefix search with deterministic lexicographic ordering
- Longest-prefix matching for autocomplete, gazetteers, dictionaries, and
  normalized entity lookup
- Bounded fuzzy matching by Unicode-aware Levenshtein distance
- Mutable operations: `insert`, `update`, `remove`, `discard`, `clear`
- UTF-8 safe keys; traversal is byte-based and keys are accepted as Python `str`
- Typed package metadata via `py.typed` and `__init__.pyi`
- Read-only frozen snapshots with bitmask child lookup inspired by `trie-hard`

## Install For Development

The project uses `maturin` and PyO3.

```bash
python3 -m venv venv
./venv/bin/python -m pip install -U pip
./venv/bin/python -m pip install -r python/requirements.txt pytest
VIRTUAL_ENV="$PWD/venv" ./venv/bin/maturin develop
./venv/bin/python -m pytest -q
```

## Build A Wheel

```bash
./venv/bin/maturin build --release
./venv/bin/python -m pip install target/wheels/trie_hard_py-*.whl
```

## Test And Benchmark

```bash
cargo test
VIRTUAL_ENV="$PWD/venv" ./venv/bin/maturin develop
./venv/bin/python -m pytest -q
./venv/bin/python benchmarks/bench_pytrie.py
```

The benchmark uses generated city, country, and medicine-style terms so prefix
and fuzzy matching resemble autocomplete and entity lookup workloads. It reports
rough RSS deltas for trie construction and writes local JSON history to
`benchmarks/results/` for regression comparisons. RSS is process-level memory, so
treat it as a coarse regression signal rather than an exact object-size
measurement.

## Usage

```python
from trie_hard_py import PyTrie

trie = PyTrie(["san antonio", "san diego", "san jose", "serbia", "sertraline"])

assert trie.get("san diego") == "san diego"
assert "serbia" in trie
assert "ser" not in trie

trie.insert("amoxicillin tablet", "RXNORM:308182")
trie["amoxicillin oral suspension"] = "RXNORM:308189"
frozen = trie.freeze()

assert list(frozen.prefix_search("amoxicillin")) == [
    ("amoxicillin oral suspension", "RXNORM:308189"),
    ("amoxicillin tablet", "RXNORM:308182"),
]
assert frozen.longest_prefix("amoxicillin tablet 500mg") == (
    "amoxicillin tablet",
    "RXNORM:308182",
)
assert frozen.fuzzy_match("sertralina", max_distance=1) == ("sertraline", "sertraline", 1)

removed = trie.remove("amoxicillin tablet")
assert removed == "RXNORM:308182"
```

## API

`PyTrie(items: list[str] | None = None)`

When constructed from strings, each key is stored with the same string as its
value.

Main methods:

- `get(key) -> str | None`
- `get_or(key, default=None) -> str | None`
- `contains(key) -> bool`
- `starts_with(prefix) -> bool`
- `prefix_contains(prefix) -> bool`
- `insert(key, value) -> str | None`
- `add(key) -> str | None`
- `update(items: list[tuple[str, str]]) -> None`
- `remove(key) -> str | None`
- `discard(key) -> bool`
- `clear() -> None`
- `keys() -> list[str]`
- `values() -> list[str]`
- `items() -> list[tuple[str, str]]`
- `prefix_search(prefix) -> Iterator[tuple[str, str]]`
- `longest_prefix(query) -> tuple[str, str] | None`
- `fuzzy_search(query, max_distance=2, limit=10) -> list[tuple[str, str, int]]`
- `fuzzy_match(query, max_distance=2) -> tuple[str, str, int] | None`
- `freeze() -> PyFrozenTrie`

Python mapping helpers are also supported: `len(trie)`, `bool(trie)`,
`key in trie`, `trie[key]`, `trie[key] = value`, `del trie[key]`, and iteration
over `(key, value)` pairs.

`PyFrozenTrie` is a read-only snapshot returned by `PyTrie.freeze()`. It supports
the read APIs (`get`, `contains`, `prefix_search`, `longest_prefix`,
`fuzzy_search`, iteration, and mapping-style reads) but does not expose mutation.

## Engineering Notes

The core is currently a mutable byte trie implemented in Rust with arena-stored
nodes and sorted compact child lists. Values are stored as shared `Arc<str>`
handles, so frozen snapshots can reuse value strings instead of copying them.
This keeps iteration deterministic without allocating a map object per node.
Deleted branches are detached from their parents; arena slots are retained so
existing node indexes stay stable.

Frozen snapshots path-compress unary chains into radix labels before applying
adaptive child lookup. Nodes with high fanout get an external 256-bit child
mask, so exact lookup checks whether the queried byte is present and computes
the child offset with `count_ones`, following the same rank-by-bitmask idea used
by Cloudflare's `trie-hard`. Low-fanout nodes avoid the fixed mask cost and use
binary search over contiguous children instead. Benchmark both mutable and
frozen forms for your workload.

The current frozen node layout intentionally keeps per-node metadata together.
An experimental structure-of-arrays layout was tested, but on the benchmark
dataset it used more RSS than the adaptive node layout, so it is not used.

It intentionally does not depend on Cloudflare's `trie-hard` crate because that
crate is optimized for bulk-loaded read-mostly maps; this package exposes
mutable Python operations.

Fuzzy search uses bounded Levenshtein distance and short-circuits candidates
that already exceed `max_distance`. Distances are counted over Unicode scalar
values rather than UTF-8 bytes.

CI builds and tests the extension on Linux, macOS, and Windows using GitHub
Actions and uploads wheel artifacts for each platform.
