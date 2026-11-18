#!/usr/bin/env python3
"""Generate large JSON files for benchmarking Unfold.

Usage:
    python3 scripts/gen_test_json.py                 # 10, 100 and 500 MB into bench-data/
    python3 scripts/gen_test_json.py 50 --out /tmp   # one 50 MB file somewhere else

The output is written as a stream, so memory use stays small even for big files.
The data is deterministic (fixed seed), so two runs give the same file and
numbers from different machines can be compared.

The shape is meant to look like a real API dump: a top level array of records
with repeated keys, nested objects, arrays of small objects, and a mix of
strings, numbers, booleans and nulls.

Generated files go in bench-data/ which is ignored by git.
"""

import argparse
import json
import os
import random
import sys

DEFAULT_SIZES_MB = [10, 100, 500]

WORDS = (
    "alpha bravo charlie delta echo foxtrot golf hotel india juliet kilo lima "
    "mike november oscar papa quebec romeo sierra tango uniform victor whiskey "
    "xray yankee zulu"
).split()
STATUSES = ["active", "pending", "suspended", "closed"]
COUNTRIES = ["ES", "CO", "CA", "US", "DE", "FR", "JP", "BR"]


def sentence(rng, n):
    return " ".join(rng.choice(WORDS) for _ in range(n))


def make_record(rng, i):
    return {
        "id": i,
        "uuid": "%08x-%04x-%04x-%04x-%012x"
        % (
            rng.getrandbits(32),
            rng.getrandbits(16),
            rng.getrandbits(16),
            rng.getrandbits(16),
            rng.getrandbits(48),
        ),
        "name": sentence(rng, 2).title(),
        "email": "user%d@example.com" % i,
        "active": rng.random() < 0.8,
        "status": rng.choice(STATUSES),
        "score": round(rng.random() * 1000, 3),
        "balance": rng.randint(-5000, 500000),
        "deleted_at": None if rng.random() < 0.9 else "2025-01-%02dT10:00:00Z" % rng.randint(1, 28),
        "address": {
            "street": "%d %s St" % (rng.randint(1, 9999), rng.choice(WORDS).title()),
            "city": rng.choice(WORDS).title(),
            "country": rng.choice(COUNTRIES),
            "geo": {"lat": round(rng.uniform(-90, 90), 6), "lng": round(rng.uniform(-180, 180), 6)},
        },
        "tags": [rng.choice(WORDS) for _ in range(rng.randint(0, 6))],
        "orders": [
            {
                "order_id": rng.getrandbits(32),
                "total": round(rng.random() * 500, 2),
                "paid": rng.random() < 0.7,
                "items": [
                    {"sku": "SKU-%05d" % rng.randint(0, 99999), "qty": rng.randint(1, 5)}
                    for _ in range(rng.randint(1, 4))
                ],
            }
            for _ in range(rng.randint(0, 4))
        ],
        "notes": sentence(rng, rng.randint(3, 14)),
    }


def write_file(path, target_bytes, seed=1983):
    rng = random.Random(seed)
    written = 0
    count = 0
    with open(path, "w", encoding="utf-8") as f:
        f.write("[\n")
        written += 2
        while True:
            chunk = json.dumps(make_record(rng, count), separators=(",", ":"))
            if count > 0:
                f.write(",\n")
                written += 2
            f.write(chunk)
            written += len(chunk)
            count += 1
            if written >= target_bytes:
                break
        f.write("\n]\n")
    return count


def main():
    parser = argparse.ArgumentParser(description="Generate large JSON files for benchmarking.")
    parser.add_argument("sizes", nargs="*", type=int, help="sizes in MB (default: 10 100 500)")
    parser.add_argument("--out", default="bench-data", help="output directory (default: bench-data)")
    args = parser.parse_args()

    sizes = args.sizes or DEFAULT_SIZES_MB
    os.makedirs(args.out, exist_ok=True)

    for mb in sizes:
        path = os.path.join(args.out, "bench-%dmb.json" % mb)
        records = write_file(path, mb * 1024 * 1024)
        actual = os.path.getsize(path) / (1024 * 1024)
        print("wrote %s: %.1f MB, %d records" % (path, actual, records))
    return 0


if __name__ == "__main__":
    sys.exit(main())
