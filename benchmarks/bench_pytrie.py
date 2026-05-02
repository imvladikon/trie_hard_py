from __future__ import annotations

import argparse
import json
import random
import statistics
import time
from dataclasses import dataclass
from collections.abc import Callable
from datetime import UTC, datetime
from pathlib import Path
from typing import Any

from trie_hard_py import PyTrie


CITY_NAMES = (
    "san antonio",
    "san diego",
    "san jose",
    "san francisco",
    "new york",
    "newark",
    "new orleans",
    "los angeles",
    "las vegas",
    "london",
    "lisbon",
    "berlin",
    "belgrade",
    "barcelona",
    "buenos aires",
    "sao paulo",
    "seoul",
    "singapore",
    "sydney",
    "tokyo",
    "toronto",
    "vancouver",
    "vienna",
    "warsaw",
    "zurich",
    "munich",
    "moscow",
    "madrid",
    "milan",
    "mexico city",
)

COUNTRY_NAMES = (
    "united states",
    "united kingdom",
    "united arab emirates",
    "canada",
    "germany",
    "france",
    "italy",
    "spain",
    "serbia",
    "brazil",
    "argentina",
    "japan",
    "south korea",
    "australia",
    "new zealand",
    "switzerland",
    "austria",
    "netherlands",
    "norway",
    "sweden",
)

MEDICINE_NAMES = (
    "amoxicillin",
    "azithromycin",
    "atorvastatin",
    "amlodipine",
    "acetaminophen",
    "ibuprofen",
    "metformin",
    "omeprazole",
    "simvastatin",
    "levothyroxine",
    "lisinopril",
    "losartan",
    "gabapentin",
    "sertraline",
    "fluoxetine",
    "cetirizine",
    "clopidogrel",
    "warfarin",
    "prednisone",
    "doxycycline",
)

ENTRY_QUALIFIERS = (
    "",
    " airport",
    " central",
    " city",
    " county",
    " province",
    " region",
    " tablet",
    " capsule",
    " oral suspension",
    " extended release",
    " pediatric",
)


def make_keys(size: int) -> list[str]:
    rng = random.Random(20260501)
    keys: set[str] = set()
    base_terms = CITY_NAMES + COUNTRY_NAMES + MEDICINE_NAMES

    while len(keys) < size:
        base = rng.choice(base_terms)
        qualifier = rng.choice(ENTRY_QUALIFIERS)
        variant = rng.randrange(1, 100_000)
        keys.add(f"{base}{qualifier} {variant:05d}".strip())

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
                name="pygtrie.CharTrie",
                build=lambda keys: build_mapping(pygtrie.CharTrie(), keys),
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


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Benchmark trie_hard_py against Python trie packages.")
    parser.add_argument("--size", type=int, default=10_000, help="Number of generated keys.")
    parser.add_argument(
        "--history-dir",
        type=Path,
        default=Path("benchmarks/results"),
        help="Directory for local JSON benchmark history.",
    )
    parser.add_argument(
        "--no-history",
        action="store_true",
        help="Run without writing or comparing local benchmark history.",
    )
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    keys = make_keys(args.size)
    lookup_hits = keys[::100]
    lookup_misses = [f"unknown city {index:05d}" for index in range(len(lookup_hits))]
    fuzzy_queries = [mutate_one_char(key) for key in keys[::1000]]

    prefix = "san "
    longest_query = f"{keys[-1]} outpatient note"
    alphabet = "".join(sorted(set("".join(keys) + "".join(lookup_misses) + prefix + longest_query)))

    print(f"dataset: {len(keys)} keys")
    print("optional competitors: marisa-trie, datrie, pygtrie, pytrie")
    print("missing optional packages are skipped")
    print()

    topology = estimate_topology(keys)
    print("topology")
    print("----------------------  -----------")
    print(f"byte trie nodes         {topology['byte_nodes']:>11}")
    print(f"radix trie nodes        {topology['radix_nodes']:>11}")
    print(f"radix node reduction    {topology['node_reduction_percent']:>10.1f}%")
    print()

    memory_rows = []
    timing_rows = []
    skipped_rows = []
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
            skipped_rows.append(
                {"name": candidate.name, "error": type(error).__name__, "message": str(error)}
            )
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

    if not args.no_history:
        write_history(args.history_dir, len(keys), topology, memory_rows, timing_rows, skipped_rows)


def estimate_topology(keys: list[str]) -> dict[str, float | int]:
    root = {"value": False, "children": {}}
    for key in keys:
        node = root
        for byte in key.encode():
            node = node["children"].setdefault(byte, {"value": False, "children": {}})
        node["value"] = True

    byte_nodes = count_nodes(root)
    radix_nodes = count_radix_nodes(root)
    reduction = 0.0 if byte_nodes == 0 else (byte_nodes - radix_nodes) * 100.0 / byte_nodes
    return {
        "byte_nodes": byte_nodes,
        "radix_nodes": radix_nodes,
        "node_reduction_percent": reduction,
    }


def count_nodes(node: dict[str, Any]) -> int:
    return 1 + sum(count_nodes(child) for child in node["children"].values())


def count_radix_nodes(node: dict[str, Any]) -> int:
    count = 1
    for child in node["children"].values():
        terminal = child
        while not terminal["value"] and len(terminal["children"]) == 1:
            terminal = next(iter(terminal["children"].values()))
        count += count_radix_nodes(terminal)
    return count


def write_history(
    history_dir: Path,
    dataset_size: int,
    topology: dict[str, float | int],
    memory_rows: list[tuple[str, int]],
    timing_rows: list[tuple[str, float, float]],
    skipped_rows: list[dict[str, str]],
) -> None:
    history_dir.mkdir(parents=True, exist_ok=True)
    latest_path = history_dir / "latest.json"
    previous = read_json(latest_path)
    timestamp = datetime.now(UTC).strftime("%Y%m%dT%H%M%SZ")
    result = {
        "timestamp": timestamp,
        "dataset_size": dataset_size,
        "topology": topology,
        "memory": [
            {"name": name, "rss_delta_bytes": rss_delta} for name, rss_delta in memory_rows
        ],
        "timings": [
            {"name": name, "mean_seconds": mean, "stdev_seconds": stdev}
            for name, mean, stdev in timing_rows
        ],
        "skipped": skipped_rows,
    }

    run_path = history_dir / f"{timestamp}.json"
    encoded = json.dumps(result, indent=2, sort_keys=True)
    run_path.write_text(encoded + "\n")
    latest_path.write_text(encoded + "\n")

    print()
    print(f"history: wrote {run_path}")
    if previous is not None:
        print_history_delta(previous, result)


def read_json(path: Path) -> dict[str, Any] | None:
    if not path.exists():
        return None
    try:
        return json.loads(path.read_text())
    except json.JSONDecodeError:
        return None


def print_history_delta(previous: dict[str, Any], current: dict[str, Any]) -> None:
    previous_memory = {
        row["name"]: row["rss_delta_bytes"] for row in previous.get("memory", [])
    }
    current_memory = {row["name"]: row["rss_delta_bytes"] for row in current.get("memory", [])}
    previous_timings = {row["name"]: row["mean_seconds"] for row in previous.get("timings", [])}
    current_timings = {row["name"]: row["mean_seconds"] for row in current.get("timings", [])}

    print("delta vs previous latest")
    for name in sorted(set(previous_memory) & set(current_memory)):
        delta = current_memory[name] - previous_memory[name]
        print(f"  memory {name:<22} {format_signed_bytes(delta):>12}")

    for name in sorted(set(previous_timings) & set(current_timings)):
        delta = current_timings[name] - previous_timings[name]
        print(f"  time   {name:<30} {delta * 1_000_000:>+10.2f} us")


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


def format_signed_bytes(value: int) -> str:
    sign = "+" if value >= 0 else "-"
    return f"{sign}{format_bytes(abs(value))}"


if __name__ == "__main__":
    main()
