"""Uncovered-base and chase intervals from a coverage trace (stdlib only).

Codex fielder-communication recheck P1-2. Make a trace with
    cd sim && BB_COVER_TRACE=1 cargo run --release -p bb_game --example coverage_trace -- 50 > trace.log 2>&1
then
    python3 analysis/coverage_gaps.py trace.log [other.log ...]

For each base: contiguous loose-ball intervals after the first assignment with no fielder
assigned to cover it (`Role::Cover(base)`), ending at a cover or when the ball is no longer
loose. These are missing jobs, not the travel time to a covered bag. Also: grounder plays with
first uncovered, intervals with no known lead, intervals with nobody chasing, and plays in
which all nine fielders start a chase more than once. Adapted from the review's own script.
"""
import collections
import json
import sys


def read(path):
    plays = collections.defaultdict(list)
    ends = {}
    digests = {}
    for line in open(path):
        if line.startswith('R,'):
            z = line.strip().split(',', 14)
            key = tuple(map(int, z[1:3]))
            plays[key].append(dict(t=float(z[3]), phase=z[5], grounder=int(z[8]), lead=int(z[7]),
                                   covers=list(map(int, z[9:13])), chasing=int(z[13]), roles=json.loads(z[14])))
        elif line.startswith('E,'):
            z = line.split(',')
            ends[tuple(map(int, z[1:3]))] = float(z[3])
        else:
            w = line.split()
            if len(w) == 2 and w[0].isdigit() and len(w[1]) == 64:
                digests[int(w[0])] = w[1]
    return plays, ends, digests


def intervals(rows, end, pred):
    start, out = None, []
    for row in rows:
        if pred(row):
            if start is None:
                start = row['t']
        elif start is not None:
            out.append((start, row['t']))
            start = None
    if start is not None:
        out.append((start, end))
    return out


def gaps(plays, ends, pred):
    vals = []
    for key, rows in plays.items():
        for a, b in intervals(rows, ends[key], pred):
            vals.append((round(b - a, 6), key, a))
    return sorted(vals, reverse=True)


def all_nine_repeats(plays):
    n = 0
    for rows in plays.values():
        starts, prev = 0, 0
        for row in rows:
            if row['chasing'] == 0x1FF and prev != 0x1FF:
                starts += 1
            prev = row['chasing']
        n += starts > 1
    return n


def report(path):
    plays, ends, digests = read(path)
    print(f'{path}: {len(digests)} games, {len(plays)} live plays')
    print('| | intervals > 0.3 s | longest |')
    print('|---|---:|---:|')
    for b, name in enumerate(['First', 'Second', 'Third']):
        v = gaps(plays, ends, lambda x, b=b: x['phase'] == 'F' and x['covers'][b] == 0)
        print(f'| {name} uncovered | {sum(d > 0.30001 for d, _, _ in v)} | {v[0][0] if v else 0:.3f} s |')
    v = gaps(plays, ends, lambda x: x['phase'] == 'F' and x['grounder'] and x['covers'][0] == 0)
    print(f'| First uncovered, grounders | {sum(d > 0.30001 for d, _, _ in v)} | {v[0][0] if v else 0:.3f} s |')
    for b, name in enumerate(['first', 'second', 'third']):
        v = gaps(plays, ends, lambda x, b=b: x['phase'] == 'F' and x['covers'][b] > 1)
        print(f'| Two covering {name} | {sum(d > 0.30001 for d, _, _ in v)} | {v[0][0] if v else 0:.3f} s |')
    v = gaps(plays, ends, lambda x: x['phase'] == 'F' and x['lead'] < 0)
    print(f'| No known lead | {sum(d > 0.30001 for d, _, _ in v)} | {v[0][0] if v else 0:.3f} s |')
    v = gaps(plays, ends, lambda x: x['phase'] == 'F' and x['chasing'] == 0)
    print(f'| Nobody chasing | {sum(d > 0.30001 for d, _, _ in v)} | {v[0][0] if v else 0:.3f} s |')
    print(f'| Plays with all nine starting a chase twice or more | {all_nine_repeats(plays)} | |')
    worst = gaps(plays, ends, lambda x: x['phase'] == 'F' and x['covers'][0] == 0)[:5]
    print('longest first-base gaps (s, (seed, play), from):', worst)


if __name__ == '__main__':
    for p in sys.argv[1:]:
        report(p)
