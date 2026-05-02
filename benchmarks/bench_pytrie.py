from __future__ import annotations

import random
import statistics
import string
import time
from dataclasses import dataclass
from collections.abc import Callable
from pathlib import Path
from typing import Any

from trie_hard_py import PyTrie


def make_keys(size: int) -> list[str]:
    alphabet = string.ascii_lowercase + string.digits
    rng = random.Random(20260501)
    keys: set[str] = set()

    while len(keys) < size:
        prefix = rng.choice(("api", "usr", "cfg", "pkg", "doc"))
        suffix = "".join(rng.choice(alphabet) for _ in range(10))
        keys.add(f"{prefix}/{suffix}")

    return sorted(keys)


def mutate_one_char(value: str) -> str:
    chars = list(value)
    for index, char in enumerate(chars):
        if char.isalnum():
            chars[index] = "z" if char != "z" else "y"
            break
    return "".join(chars)


def measure(name: str, iterations: int, fn: Callable[[], object]) -> tuple[str, float, float]:
    samples = []
    for _ in range(3):
        start = time.perf_counter()
        for _ in range(iterations):
            fn()
        elapsed = time.perf_counter() - start
        samples.append(elapsed / iterations)

    return name, statistics.mean(samples), statistics.stdev(samples)


@dataclass
class Candidate:
    name: str
    build: Callable[[list[str]], Any]
    exact_hit: Callable[[Any, list[str]], object]
    exact_miss: Callable[[Any, list[str]], object]
    prefix_search: Callable[[Any, str], object] | None = None
    longest_prefix: Callable[[Any, str], object] | None = None
    fuzzy: Callable[[Any, list[str]], object] | None = None


def available_candidates(alphabet: str) -> list[Candidate]:
    candidates = [
        Candidate(
            name="dict",
            build=lambda keys: {key: key for key in keys},
            exact_hit=lambda obj, keys: [obj.get(key) for key in keys],
            exact_miss=lambda obj, keys: [obj.get(key) for key in keys],
        ),
        Candidate(
            name="PyTrie",
            build=lambda keys: PyTrie(keys),
            exact_hit=lambda obj, keys: [obj.get(key) for key in keys],
            exact_miss=lambda obj, keys: [obj.get(key) for key in keys],
            prefix_search=lambda obj, prefix: list(obj.prefix_search(prefix)),
            longest_prefix=lambda obj, query: obj.longest_prefix(query),
            fuzzy=lambda obj, queries: [obj.fuzzy_match(query, max_distance=1) for query in queries],
        ),
        Candidate(
            name="PyFrozenTrie",
            build=lambda keys: PyTrie(keys).freeze(),
            exact_hit=lambda obj, keys: [obj.get(key) for key in keys],
            exact_miss=lambda obj, keys: [obj.get(key) for key in keys],
            prefix_search=lambda obj, prefix: list(obj.prefix_search(prefix)),
            longest_prefix=lambda obj, query: obj.longest_prefix(query),
            fuzzy=lambda obj, queries: [obj.fuzzy_match(query, max_distance=1) for query in queries],
        ),
    ]

    try:
        import marisa_trie
    except ImportError:
        pass
    else:
        candidates.append(
            Candidate(
                name="marisa.StringTrie",
                build=lambda keys: marisa_trie.StringTrie((key, key) for key in keys),
                exact_hit=lambda obj, keys: [obj.get(key) for key in keys],
                exact_miss=lambda obj, keys: [obj.get(key) for key in keys],
                prefix_search=lambda obj, prefix: [(key, obj[key]) for key in obj.keys(prefix)],
            )
        )

    try:
        import datrie
    except ImportError:
        pass
    else:
        candidates.append(
            Candidate(
                name="datrie.Trie",
                build=lambda keys: build_datrie(datrie, alphabet, keys),
                exact_hit=lambda obj, keys: [obj.get(key) for key in keys],
                exact_miss=lambda obj, keys: [obj.get(key) for key in keys],
                prefix_search=lambda obj, prefix: obj.items(prefix),
                longest_prefix=lambda obj, query: obj.longest_prefix_item(query),
            )
        )

    try:
        import pygtrie
    except ImportError:
        pass
    else:
        candidates.append(
            Candidate(
                name="pygtrie.StringTrie",
                build=lambda keys: build_mapping(pygtrie.StringTrie(), keys),
                exact_hit=lambda obj, keys: [obj.get(key) for key in keys],
                exact_miss=lambda obj, keys: [obj.get(key) for key in keys],
                prefix_search=lambda obj, prefix: list(obj.iteritems(prefix=prefix)),
                longest_prefix=lambda obj, query: obj.longest_prefix(query),
            )
        )

    try:
        import pytrie
    except ImportError:
        pass
    else:
        trie_type = getattr(pytrie, "SortedStringTrie", getattr(pytrie, "StringTrie", None))
        if trie_type is not None:
            candidates.append(
                Candidate(
                    name=f"pytrie.{trie_type.__name__}",
                    build=lambda keys, trie_type=trie_type: build_mapping(trie_type(), keys),
                    exact_hit=lambda obj, keys: [obj.get(key) for key in keys],
                    exact_miss=lambda obj, keys: [obj.get(key) for key in keys],
                    prefix_search=lambda obj, prefix: list(obj.iteritems(prefix=prefix)),
                    longest_prefix=lambda obj, query: obj.longest_prefix(query),
                )
            )

    return candidates


def build_mapping(obj: Any, keys: list[str]) -> Any:
    for key in keys:
        obj[key] = key
    return obj


def build_datrie(datrie_module: Any, alphabet: str, keys: list[str]) -> Any:
    trie = datrie_module.Trie(alphabet)
    for key in keys:
        trie[key] = key
    return trie


def benchmark_candidate(
    candidate: Candidate,
    keys: list[str],
    lookup_hits: list[str],
    lookup_misses: list[str],
    prefix: str,
    longest_query: str,
    fuzzy_queries: list[str],
) -> tuple[int, list[tuple[str, float, float]]]:
    rss_before = current_rss_bytes()
    obj = candidate.build(keys)
    rss_after = current_rss_bytes()

    assert len(obj) == len(keys)
    assert all(candidate.exact_hit(obj, lookup_hits))
    assert candidate.exact_miss(obj, lookup_misses).count(None) == len(lookup_misses)

    rows = [
        measure(f"{candidate.name} exact hit", 100, lambda: candidate.exact_hit(obj, lookup_hits)),
        measure(
            f"{candidate.name} exact miss",
            100,
            lambda: candidate.exact_miss(obj, lookup_misses),
        ),
    ]

    if candidate.prefix_search is not None:
        prefix_result = candidate.prefix_search(obj, prefix)
        assert len(prefix_result) == sum(key.startswith(prefix) for key in keys)
        rows.append(
            measure(
                f"{candidate.name} prefix",
                10,
                lambda: candidate.prefix_search(obj, prefix),
            )
        )

    if candidate.longest_prefix is not None:
        assert candidate.longest_prefix(obj, longest_query)
        rows.append(
            measure(
                f"{candidate.name} longest",
                100,
                lambda: candidate.longest_prefix(obj, longest_query),
            )
        )

    if candidate.fuzzy is not None:
        assert all(candidate.fuzzy(obj, fuzzy_queries))
        rows.append(
            measure(
                f"{candidate.name} fuzzy d=1",
                10,
                lambda: candidate.fuzzy(obj, fuzzy_queries),
            )
        )

    return rss_after - rss_before, rows


def main() -> None:
    keys = make_keys(10_000)
    lookup_hits = keys[::100]
    lookup_misses = [f"missing/{index:05d}" for index in range(len(lookup_hits))]
    fuzzy_queries = [mutate_one_char(key) for key in keys[::1000]]

    prefix = "api/"
    longest_query = f"{keys[-1]}/tail"
    alphabet = "".join(sorted(set("".join(keys) + "".join(lookup_misses) + prefix + longest_query)))

    print(f"dataset: {len(keys)} keys")
    print("optional competitors: marisa-trie, datrie, pygtrie, pytrie")
    print("missing optional packages are skipped")
    print()

    memory_rows = []
    timing_rows = []
    for candidate in available_candidates(alphabet):
        try:
            rss_delta, rows = benchmark_candidate(
                candidate,
                keys,
                lookup_hits,
                lookup_misses,
                prefix,
                longest_query,
                fuzzy_queries,
            )
        except Exception as error:
            print(f"skip {candidate.name}: {type(error).__name__}: {error}")
            continue

        memory_rows.append((candidate.name, rss_delta))
        timing_rows.extend(rows)

    print("memory")
    print("----------------------  -----------  -----------")
    for name, rss_delta in memory_rows:
        print(f"{name:<22} {format_bytes(rss_delta):>11} {rss_delta / len(keys):>9.1f} B/key")

    print()
    print("benchmark                mean/run       stdev")
    print("----------------------  -----------  -----------")
    for bench_name, mean, stdev in timing_rows:
        print(f"{bench_name:<30} {mean * 1_000_000:>9.2f} us {stdev * 1_000_000:>9.2f} us")


def current_rss_bytes() -> int:
    statm = Path("/proc/self/statm")
    if statm.exists():
        pages = int(statm.read_text().split()[1])
        return pages * 4096

    import resource

    # Linux reports ru_maxrss in KiB; macOS reports bytes. This fallback is a
    # coarse peak-RSS value when current RSS is unavailable.
    return resource.getrusage(resource.RUSAGE_SELF).ru_maxrss * 1024


def format_bytes(value: int) -> str:
    units = ("B", "KiB", "MiB", "GiB")
    amount = float(value)
    for unit in units:
        if abs(amount) < 1024 or unit == units[-1]:
            return f"{amount:.2f} {unit}"
        amount /= 1024
    return f"{amount:.2f} GiB"


if __name__ == "__main__":
    main()
