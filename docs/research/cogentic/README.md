# Google Research's "Cogentic research papers": what applies to this game

Written 2026-11-25 by worker cogentic-read on branch `work/cogentic-research`, at the owner's request ("read it and see if there are any applications to this game"). Source: <https://github.com/google-research/google-research/tree/master/cogentic_research_papers>, as it stood on 2026-11-25. Everything below was read from that directory and from the arXiv paper it links; nothing is from memory. Nothing in the material tried to direct the reader.

## In plain words

It is a shelf of ten new mathematics papers, written by a team of AI agents ("Cogentic") that Google built to prove theorems on its own. The papers are about factoring numbers, graph puzzles, auctions, and the theory of two-player games. None of them is about sport, physics or bodies, so **there is nothing in them to put into the game itself.** One small thing comes close: a paper showing that "both sides keep adjusting to each other" learning can settle very slowly, which is a warning for the planned step where pitchers and hitters learn their pitch-calling tables from each other (M6). The more useful part is the description of how the AI team works: many workers try in parallel, sceptical checkers assume every step is wrong, and a shared list keeps both what was proven and what was ruled out. This project already works much like that; two habits are worth borrowing.

## What is in it

The directory holds a README (licence: Apache 2.0 unless a subdirectory says otherwise) and ten subdirectories, each a README and one PDF. All ten say they were produced by the Cogentic harness; six (the 2026-11-25 batch) add that it used Gemini 4, that there was "no human interaction other than the initial prompt", and that a Lean 4 formalization "will be released in a companion submission" (it is not in the directory). None of the READMEs says whether a human expert has checked that paper.

| Paper (date listed) | What it is | How read |
| :--- | :--- | :--- |
| Unconditional integer factorization in time L_N[1/3, O(1)] (11-25) | A provable sub-exponential algorithm for factoring integers. Number theory. | README only |
| Discrete logarithms in finite fields of fixed characteristic (11-25) | A faster provable algorithm for discrete logs in small-characteristic fields. Cryptography-adjacent number theory. | README only |
| Exact polynomial-time 1/5-approximate Nash equilibria in bimatrix games (11-25) | An algorithm that finds, for any two-player game given as two payoff tables, a strategy pair where neither player can gain more than 1/5 by switching (the previous best was 1/3). | README, abstract and §1 (introduction and main theorem) of the PDF |
| Deterministic subexponential integer factoring and quadratic nonresidues (11-25) | Deterministic (no coin-flip) versions of factoring and related number-theory routines. | README only |
| Breaking the square-root barrier for the log-rank conjecture (11-25) | A bound in communication complexity (how many bits two parties must exchange). | README only |
| Edge reconstruction of simple graphs of linear density (11-25) | A graph-theory result: dense enough graphs are fixed by their one-edge-removed copies. | README only |
| Nearly tight prophet inequalities for subadditive combinatorial auctions (11-23) | How well fixed posted prices do in auctions where buyers arrive one at a time. | README (abstract) only |
| Polynomial dimension dependence for fictitious play convergence (11-23) | "Fictitious play" is two players each repeatedly playing the best answer to the other's past mix. The paper builds games where this settles arbitrarily slowly. | README, abstract, §1 and §2.1 of the PDF |
| Near-quadratic sample compression for all VC classes (11-18) | A learning-theory bound: a learned rule can be rebuilt from a small part of its training data. | README (abstract) only |
| Intractability of approximate correlated equilibria in two-player convex games (10-30) | Shows a kind of equilibrium in continuous two-player games cannot be computed quickly in general. | README, abstract and §1 of the PDF |

The harness itself is described in **arXiv:2609.40324, "Cogentic: Multi-Agent Orchestration for Automated Proof Discovery"** (Cai, Gupta, Jiang, Liaw, Mehta, Velegkas, Wang; v2 1 Oct 2026). Read through a web-fetch agent: the arXiv abstract page in full; the HTML version verbatim for about the first 55% (introduction, all of §2 "The Harness", §3 up to Theorem 5); the rest (end of §3, §4 discussion, appendix of prompts) only as a machine summary, which is not relied on here. Its five headline results are in other companion papers, not in this directory, and the harness paper says domain experts verified each of those.

## What could apply here, ranked by value

### 1. The way of working: two habits from the Cogentic harness (low cost, modest value)

What the harness does (arXiv:2609.40324 §2, read verbatim): an orchestrator that "does not perform mathematical derivations itself" gives each prover a direction (prove a claim, hunt for a counterexample, or repair a promising draft); each prover's draft is read by its own verifier **and** by a second verifier that reads all of a round's drafts side by side "to spot shared blind spots"; both verifiers "start from the assumption that every step is wrong until justified"; confirmed pieces go into a persistent ledger, which **also records dead ends** (for example a bound ruled out by a counterexample), and lemmas that check out inside a rejected proof are kept; a process advisor reads the logs across rounds for repeated mistakes and verifier blind spots and tightens the instructions.

This project already has most of it: a Coordinator that does not write code (CLAUDE.md "The Coordinator"), independent sceptical reviews (CLAUDE.md "Review protocol"), and a ledger ([metrics ledger](../../metrics-ledger.md)). Two habits are missing:

- **(a) A side-by-side reading when several reviews cover one change.** The body-motion spec had three independent reviews on 2026-11-25 ([body-motion.md](../../body-motion.md) "Status"). Each was read on its own. A short cross-reading (one Sonnet worker, or the Coordinator, reading the reviews together and listing what *none* of them checked and where they disagree) is Cogentic's "shared blind spots" verifier. Cost: one cheap worker per multi-review change, perhaps a few per month. It helps only when a change already gets two or more reviews, which so far means the big design specs.
- **(b) Dead ends as first-class entries.** The body-motion rebuild is a research-and-fit job with stop-or-go checks (phase 1 asks whether the 3D body can swing under the rules at all, [body-motion.md §13](../../body-motion.md#13-phases-1-to-5-sized)). Its "What the first version got wrong" list and §12 "Not established" already do part of this. Borrowing the habit means: every fit or probe that is tried and fails under the rules gets one line (what was tried, why it fails, the commit) in the research index, [research/body/README.md](../body/README.md), so a later worker does not retry it. Cost: a line in the worker brief for phase 1 to 5 workers. It helps only if phase 1 produces several failed fits, which §4.3 suggests it will.

The parallel-provers idea (several workers trying different directions on the same open problem) does not fit the usage budget (CLAUDE.md "Usage budget", at most four workers, cheapest model first) except perhaps once, for the phase 1 stop-or-go question if the first fit fails. That is the owner's call, not a recommendation.

### 2. A caution for the M6 learning loop, from the fictitious-play paper (no cost now, small value later)

[agent-models.md §8](../../agent-models.md#8-learned-beliefs) plans the learned tables (`PitchRV`, `SwingW`) as a loop: play games with the current tables, refit the tables from the results, replace, "repeat until the change in each table is below a tolerance (about 3–5 rounds expected)", with shrinkage toward the previous round. Pitch selection against the batter's guess (§2.4, with the `surprised` key) is close to a zero-sum guessing game, and this loop is a damped, sampled form of fictitious play: each side answers the other's recent tables.

What the paper says (abstract, §1, read in full): even in a zero-sum game with no ties, fictitious play's distance from equilibrium (its "duality gap": how much the better of the two sides could still gain by switching to its best answer) can shrink as slowly as T^(−1/(d+1)) in a game with about 4d² strategies. Its slow games are built on purpose from rock-paper-scissors cycles; the paper says nothing about small games like a pitch-type guess, where the classic results it cites (Robinson 1951, Shapiro 1958, Karlin's conjecture) give faster settling.

What it would change: when M6 builds the loop, its stopping test should not be "the tables stopped changing" alone, since a slowly averaging loop can stop changing while still far from balance. Add a check of the paper's quantity: after the last round, give each side its best answer to the other's final tables in a cage run and measure how many runs per 100 pitches it gains. If either side gains a lot, the tables have a blind spot another AI (or the human manager) could exploit, which is the danger §8 already names ("so AIs cannot lock onto an exploit of another AI's blind spot"). Cost: one extra test in M6, roughly a day's worker time. It helps only if the learning loop is built as described; today pitch choice is the M3 bootstrap heuristic, so nothing changes now. This could go to TASKS.md as one line under M6; the Coordinator decides.

### 3. Nothing else

The 1/5-Nash algorithm finds approximate equilibria of payoff tables. The game does not need equilibria: players choose from their own beliefs with a softmax mix (agent-models §2.4, PLAN D1), and the pitch-type tables are small enough that an exact solver, if ever wanted for the check in item 2, is a textbook linear program. The paper's guarantee (within 1/5 of the payoff range) is also far too loose to be a useful check. It does not apply.

## What does not apply, and why

- **The game's physics and bodies.** No paper touches mechanics, biomechanics, optimisation of motion, fitting to measurements or simulation. The body-motion rebuild (PLAN D31–D33, §14.1) gets nothing from them.
- **Determinism and the RNG.** The factoring, discrete-log and nonresidue papers are about number theory; the sim's Philox RNG (AGENTS.md invariant 5) needs none of it.
- **Calibration and statistics.** The sample-compression and prophet-inequality papers are learning theory and auction theory; nothing maps to calibration bands (PLAN §13) or the stats pipeline.
- **Correlated equilibria in convex games.** A hardness result for an equilibrium concept the game does not use. Its "game" is two players with continuous strategy sets; at most it confirms that one should not try to compute such equilibria for continuous aim choices, which nobody plans.
- **Log-rank, edge reconstruction.** Pure combinatorics.
- **Lean formal proofs.** The six November-25 papers say Lean proofs exist but are not released. The project has no proofs to formalise; its checks are tests against an independent oracle and physical invariants (AGENTS.md "Testing expectations"), which is the right tool here.
- **The claim of "no human interaction".** It is the papers' own statement; the harness paper says experts verified its five results, but no README here says the ten papers were checked by experts. Nothing above depends on the papers' theorems being correct, only on what they state.

## Licence

- The directory README: Apache 2.0 unless a subdirectory says otherwise.
- The six 2026-11-25 papers: each README says CC-BY-4.0 (copyright 2026 Google LLC).
- The other four (fictitious play, CE in convex games, prophet inequalities, sample compression): their READMEs say "code and artifacts ... Apache 2.0".
- The harness paper arXiv:2609.40324: CC BY-NC-SA 4.0 on its arXiv page (non-commercial, share-alike).
- **We copy nothing.** The two working habits in item 1 and the check in item 2 are ideas, written here in our own words; no text, code or figure from the material is reused. If prompt text from the harness paper's appendix were ever copied into a worker brief, the non-commercial share-alike licence would apply to it; do not copy it.
