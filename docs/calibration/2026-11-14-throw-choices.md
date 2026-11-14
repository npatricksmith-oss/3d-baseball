# A4: throw-choice evidence (2026-11-14)

## Measurement contract

This is the measuring half of owner backlog A4, from `a69dd50`. No option,
weight, timing estimate, random stream, or game fact is changed. The reference
is agent-models §5.5–5.7 and the owner's two-out example: take an available
force at first or second rather than a less secure tag at home.

The diagnostic will opt into a thread-local, read-only decision trace. It copies
the holder's existing calculations at the point he commits to a throw or carry,
including nearby carries that bypass the weighted throw loop. Consecutive carry
ticks toward the same base in the same possession are one commitment, with a
first/last time and tick count. Unselected alternatives are estimates, never
counterfactual realized outcomes. Score, inning, visible runners and eventual
facts are diagnostic context only; none flows back into a decision.

`throw_choice_probe` will run several hundred deterministic games, emit optional
JSONL for each commitment, classify conservative choice flags from the estimates,
and identify seed/play/time for watching. Rates count commitments (classes can
overlap); failed execution alone is not a wrong choice. Every measured seed is
also run with tracing disabled and its event-log digest compared. A pre-hook
digest sample guards against a change shared by the enabled and disabled paths.

Results, classification thresholds, limitations and ranked proposals follow after
measurement. Proposed changes are recommendations only.
