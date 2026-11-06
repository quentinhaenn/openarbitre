# Recalibration measures after ST-04 (reconstructed)

Input for § "Recalibrage après ST-04 (F-24, F-25)" of `04_plan.md`. The G-4 timestamps (`debut`, `fin`, `portes`) were never written, so condition F-40 is not met as specified. Every number below is **reconstructed after the fact** from Claude Code transcripts and git/GitHub timestamps. Written 2026-11-06. This file does not date the plan: that is Quentin's call.

## Data sources

- Claude Code transcripts in `~/.claude/projects/-Users-quentin-pro-Perso-openarbitre/`: 18 main sessions, plus 63 subagent transcripts (`<session>/subagents/*.jsonl`, each with a `.meta.json` description). Only `user` and `assistant` entries are used.
- `git log --all` (commit times) and `gh pr list --state merged` (PR creation and merge times).
- `.ctdd/stories/ST-0{1..4}/state.json` for phase names. They hold no timestamps except `started_at` and, for ST-01 only, `closed_at`.
- All times below are Paris local time (UTC+1).

## Method and assumptions

1. **Human input** is a `user` entry that is plain text typed by Quentin (including slash commands that launch a phase agent, `/clear`, and "[Request interrupted]"), or the answer to an `AskUserQuestion` tool call. These do not count as human input: tool results, `<task-notification>`, subagent hand-backs ("Another Claude session sent a message"), compaction summaries, local-command output. Permission prompts are not visible in transcripts and are not counted.
2. **Story attribution.** Each story is a stream of events:
   - **Subagents:** attributed by their description (`ST-02 phase 6 skeleton`, `Fix ST-02 review findings`, and so on). Two cases with no story id in the description were attributed by content: `Rename write::Value to FieldValue` went to ST-03, the foundation amendment made during its implementation (commit 6894d8e).
   - **Main-thread events:** attributed by time window, set from commits, PR merges and the topic of the conversation:

     | Story | Main-thread windows | Sessions |
     |---|---|---|
     | ST-01 | 11-03 00:40 → 11-03 23:05 (PR #1 merged) · 11-04 12:53 → 12:58 (closure, PR #2) | orchestrator `0c225d8e` and the phase sessions `1ad88d62`, `1e7ca27f`, `cd28d6aa`, `3da2ea78`, `7be6d832`, `343001f3`, `48aacda7` (one per `/story-writer`, `/specificateur`, `/architecte`, `/planner`, `/devil-advocate` run) |
     | ST-02 | 11-04 12:58 → 21:38 · 21:45 → 11-05 00:41 · 11-05 10:37 → 12:28 (PR #6 merged 12:27) | `803c7fed` |
     | ST-03 | 11-04 21:41 → 11-05 00:41 · 11-05 12:28 → 12:52 (merge of `main` into `st-03/story`). PR #8 merged at 16:04 with no transcript activity | `803c7fed` |
     | ST-04 | 11-05 11:26 → 12:28 · 12:52 → 20:03 (PR #10 merged 20:01) | `803c7fed` |

   - **Human messages in `803c7fed`:** attributed one by one by their content, for example "Q-R1 : b" → ST-02 (commit 884fc15) and "ok pour a partout" → ST-04 Q1–Q7. One message can serve two stories (11-04 21:51 answers ST-02's Q-R1 and ST-03's Q1–Q8).
   - **Overlaps are counted in both stories.** ST-02 ‖ ST-03 (11-04 21:45 → 00:41) and ST-02 ‖ ST-04 (11-05 11:26 → 12:28) ran in parallel. Each story keeps its own elapsed time, because the plan's unit is the duration of a cycle in its lane. Σ(m + a) = 1059 min, while the active wall time of the four streams together is 867 min. About 190 min is therefore counted twice.
3. **Gate** = from the last main-thread assistant event in the story's stream to the human message. With phase sessions (ST-01), that is the last assistant event in any of the parallel sessions. **a** = Σ gates. **m** = active span − a. Subagents working in the background during a gate do not shorten it.
4. **Pauses.** Any gap between two consecutive events of the stream longer than T is removed from both a and m, whether it is a gate or machine time. Primary value: T = 30 min. Sensitivity: T = 15 min and T = 60 min.
5. **Phases** are given by the latest phase marker: the launch of a phase session or a phase subagent. A gate belongs to the phase whose output it arbitrates. Phases 4 and 5 (plan ⇄ DA loop) are merged. ST-03 skipped phase 3. ST-04 ran phases 1–5 in one subagent and 6–7 together.

### Excluded (listed, not measured)

- **Global framing cycle (cadre)**, 11-02 21:14 → 11-03 00:40: the orchestrator `0c225d8e` up to ST-01's `started_at`, and sessions `f371b8cc`, `6494b67d`, `d34a6520`, `9a9db913`, `2b5d715b`, `a2ecf30b`, `5c08afca`, `7a56d9ab`, `b68577f4`.
- **Spikes:** S1-A (subagent, 11-03 01:32–01:39), S4, S5, S4-R (subagents, 11-04 21:42–21:59), and the Linear research subagent (11-04 22:09).
- **Off-topic spans inside `803c7fed`:**
  - node keychain question, 11-04 12:50–12:53;
  - backlog BL-1 and PR graph, 18:36–18:42;
  - "when is it testable", N = 3 and the spikes discussion, 21:38–21:45;
  - Linear watch, 22:07–22:12;
  - anti-pattern discussion, 22:41–22:43.
- **Debt work:** the whole-repo ponytail audit and debt PRs #11–#14 (`803c7fed` after 20:03 on 11-05, and `aed15546`), and `3b53ae65` (`/exit` only).
- **Counted inside a story:** PR #5 (CI amendment, ST-02 T-1 subagent) and PR #7 (parallel mutants and lint, discussed during ST-02/ST-03 in the shared window). Both are foundation amendments made inside those cycles.

## Per-story results (T = 30 min)

Estimates from `04_plan.md`: all four are M (100 min). ST-01 and ST-03 are rated `forte` (4 points = 60 min), ST-02 and ST-04 `moyenne` (2 points = 30 min). Σ taille = 400, Σ arbitrage = 180.

| Story | Est. taille / arb. | **m** | **a** | gates | m / taille | a / arb. | m at T=15 / 60 | a at T=15 / 60 |
|---|---|---|---|---|---|---|---|---|
| ST-01 | 100 / 60 | **129** | **141** | 68 | 1.29 | 2.35 | 129 / 129 | 115 / 309 |
| ST-02 | 100 / 30 | **255** | **106** | 17 | 2.55 | 3.53 | 238 / 327 | 46 / 252 |
| ST-03 | 100 / 60 | **134** | **34** | 5 | 1.34 | 0.57 | 134 / 168 | 34 / 34 |
| ST-04 | 100 / 30 | **239** | **21** | 4 | 2.39 | 0.70 | 211 / 239 | 21 / 21 |
| **Σ** | 400 / 180 | **758** | **301** | 94 | | | 712 / 864 | 216 / 616 |

Per phase, m / a in minutes (T = 30). Phase 8 includes PR, CI, rebase and merge handling.

| Story | 1–2 story, specs | 3 archi | 4–5 plan ⇄ DA | 6 skeleton | 7 implementation | 8 review, fixes, merge |
|---|---|---|---|---|---|---|
| ST-01 | 18 / 54 | 8 / 4 | 40 / 32 | 10 / 6 | 17 / 10 | 37 / 35 |
| ST-02 | 11 / 39 | 9 / 16 | 29 / 0 | 8 / 0 | 51 / 19 | 147 / 33 |
| ST-03 | 11 / 7 | skipped | 8 / 15 | 12 / 7 | 29 / 5 | 74 / 0 |
| ST-04 | 45 / 8 (phases 1–5) | — | — | 112 / 0 (phases 6–7) | — | 83 / 12 |

## Factors

| | T = 15 | **T = 30** | T = 60 | Plan estimate |
|---|---|---|---|---|
| k_m = Σm / 400 | 1.78 | **1.89** | 2.16 | 1 |
| k_a = Σa / 180 | 1.20 | **1.67** | 3.42 | 1 |
| f = Σa / Σ(m + a) | 0.23 | **0.28** | 0.42 | 0.31 |

The threshold has little effect on k_m (1.8–2.2) and a large effect on k_a (1.2–3.4). Long gates are where the threshold cuts, and most long gates end with a human message.

## Step 3: ceiling (B = 120, the declared default)

| | T = 15 | **T = 30** | T = 60 | Plan estimate |
|---|---|---|---|---|
| 150 × f | 34.9 | **42.7** | 62.4 | 46.5 |
| N = min(3, max(1, ⌊B / 150f⌋)) | 3 | **2** (⌊2.81⌋) | 1 | 2 |
| Budget = B / (15 × k_a), points per evening | 6.7 | **4.8** | 2.3 | 8 |

With a budget of 4.8, an evening holds one `forte` element (4 points) and nothing else above 0.8, or two `moyenne` elements. The budget, not N, is the binding constraint. At T = 60 the budget is 2.3, below the 4 points of one `forte` element.

## Step 4: recalibrated durations (T = 30)

Recalibrated duration = 1.89 × taille + 1.67 × 15 × points. Taille: S = 50, M = 100, L = 150. G-3 has no taille, so only its human part (6 points) is scaled.

| Lane P | Plan | Recal. | Lane D | Plan | Recal. | Lane M | Plan | Recal. |
|---|---|---|---|---|---|---|---|---|
| ST-05 (M, 2) | 130 | 240 | ST-22 (L, 4) | 210 | 385 | ST-04b (M, 2) | 130 | 240 |
| ST-06 (S, 1) | 65 | 120 | ST-23 (M, 2) | 130 | 240 | ST-18 (S, 2) | 80 | 145 |
| ST-07 (M, 2) | 130 | 240 | ST-24 (M, 2) | 130 | 240 | ST-19 (L, 4) | 210 | 385 |
| ST-08 (S, 1) | 65 | 120 | ST-25 (M, 4) | 160 | 290 | S1-B + S2 (M, 4) | 160 | 290 |
| ST-09 (M, 2) | 130 | 240 | G-3 (—, 6) | 90 | 151 | ST-20 (L, 4) | 210 | 385 |
| S6 (S, 1) | 65 | 120 | ST-26 (L, 2) | 180 | 334 | ST-21 (M, 2) | 130 | 240 |
| ST-11 (M, 2) | 130 | 240 | ST-27 (M, 2) | 130 | 240 | | | |
| ST-12 (M, 1) | 115 | 215 | ST-28 (M, 2) | 130 | 240 | | | |
| ST-13 (M, 2) | 130 | 240 | ST-29 (S, 1) | 65 | 120 | | | |
| ST-14 (M, 4) | 160 | 290 | ST-30 (M, 2) | 130 | 240 | | | |
| ST-15a (M, 2) | 130 | 240 | ST-31 (S, 1) | 65 | 120 | | | |
| ST-16 (M, 2) | 130 | 240 | ST-32 (L, 2) | 180 | 334 | | | |
| S3-V (S, 3) | 95 | 170 | | | | | | |
| ST-10 (M, 2) | 130 | 240 | | | | | | |
| ST-15b (M, 2) | 130 | 240 | | | | | | |
| *ST-17 fallback (L, 4)* | *210* | *385* | | | | | | |
| **Σ P (Google)** | 1735 | 3195 | **Σ D** | 1600 | 2934 | **Σ M** | 920 | 1685 |

## Step 5: milestone evenings

A script applies opening rules 1–7 one element at a time. The script is kept outside the repo. Choices made where the plan leaves room:

- **Evening.** One evening = 150 min. Evening 1 is the first evening after the recalibration.
- **Opening order.** ST-22 opens first (PL-9). After that, P has priority over D, and D over M.
- **Cross-lane edges.** A lane whose next element waits on an edge from another lane gives its place to the next lane (ST-07 → ST-27, ST-32; ST-11 → ST-19; ST-06, ST-08, ST-13, ST-16 → ST-20, ST-21).
- **Rules 1–3.** A `faible` element is not a cycle and does not hold a place. At most N cycles run at once, and at most one `forte`.
- **Points.** An element's points count in the evening it opens. Elements open while that evening's total stays within the budget, but an element may always open alone in an evening, otherwise nothing could open at T = 60.
- **S3.** The default path (S3-V → ST-10 → ST-15b) assumes the admin has answered by the time ST-16 merges. The CalDAV row runs the fallback (ST-17).
- **Dates.** Evening k is mapped to dates from the week of 2026-11-09, at 2 evenings a week (Tuesday and Thursday) or 3 a week (Monday, Wednesday and Friday). The weekdays are an assumption: only the rhythm comes from the plan.

| Milestone | End of | Evening (T=30) | 2 / week | 3 / week | Range T=15 … T=60 | Plan defaults (k = 1, N = 2, 8 pts) |
|---|---|---|---|---|---|---|
| Decision diff demo ("3 violations") | ST-22 | **3** | 2026-11-17 | 2026-11-13 | 3 … 4 | 2 |
| Decision diff in CI | ST-24 | **7** | 2026-12-01 | 2026-11-23 | 6 … 14 | 4 |
| Planning on CLI and assistant, simulated calendar | ST-09 | **9** | 2026-12-08 | 2026-11-27 | 6 … 16 | 4 |
| Morning recompute, tool closed, simulated | ST-11 | **12** | 2026-12-17 | 2026-12-04 | 8 … 21 | 5 |
| Sync between two computers | ST-19 | **15** | 2026-12-29 | 2026-12-11 | 12 … 59 | 7 |
| Decision diff complete | ST-32 | **22** | 2027-01-21 | 2026-12-28 | 18 … 56 | 13 |
| Planning complete on simulator | ST-16 | **23** | 2027-01-26 | 2026-12-30 | 16 … 33 | 11 |
| Planning on real calendar, CalDAV fallback | ST-17 | **25** | 2027-02-02 | 2027-01-04 | 19 … 36 | 12 |
| **Planning on real calendar, Google** | ST-15b | **27** | 2027-02-09 | 2027-01-08 | 20 … 39 | 13 |
| Phone: read | ST-20 | **27** | 2027-02-09 | 2027-01-08 | 19 … 66 | 14 |
| Phone: act (last element) | ST-21 | **29** | 2027-02-16 | 2027-01-13 | 21 … 68 | 15 |

Dates for the bounds of the range, at 3 evenings a week: T = 15 ends on 2026-12-25 (evening 21), and T = 60 on 2027-04-14 (evening 68). At 2 evenings a week, they are 2027-01-19 and 2027-07-01.

One side effect of rule 1 shows in this run. ST-12 is `faible`, so it is not a cycle. While lane P runs it, lane M takes the free place (ST-19, evening 13), and ST-13 then waits for that place.

## Step 6: S3 (reported, not decided)

`.ctdd/cadre/state.json` → `gestes_en_attente.S3-L` says: "Client OAuth Google perso + demande admin Workspace, à faire par Quentin le 2026-11-03 ; la date de demande fixe la borne de S3". Nothing in the repo records that S3-L was done, nor any answer from the admin.

## Reliability

**Measured** (from timestamps, to the second):
- event times in the transcripts and subagents;
- human message times;
- commit and PR merge times.

**Reconstructed** (judgement calls that change the numbers):
- **Attribution.** The main-thread windows and the story assigned to each human message in `803c7fed` are manual. Five off-topic spans were cut by hand.
- **Overlaps counted twice.** About 190 of the 1059 minutes come from overlaps counted in both stories. If those minutes were split instead, k_m would fall to about 1.4.
- **Gate start.** A gate opens at the last main-thread assistant event, not at the moment an agent actually asked Quentin a question. If the orchestrator posted a status line while Quentin was reading, the gate is shortened.
- **Arbitration lost in pauses.** A gate longer than T counts as a pause, so Quentin's reading time inside it is lost. This affects:
  - ST-02's plan/DA arbitration, inside a 233-min pause;
  - ST-01's archi and specs reviews, in 34–133-min gaps;
  - the two merge gestures of ST-04, of 120 and 137 min.

  This is why k_a moves from 1.2 to 3.4 across thresholds.
- **Long machine runs dropped.** Background CI waits and long mutation runs with no event for more than T are dropped. One example is the 37 min run in "Apply ST-03 review fixes".
- **Not evenings.** The work happened partly in the daytime: 11-04 from 12:52 and 11-05 from 10:37 to 20:02. The pause rule is the only stand-in for "out of evening". The socle used about 867 min of active wall time over 3 days (265, 331 and 271 min). The plan had 4 evenings of 150 min.
- **Process change between stories.** ST-01 ran with phase agents launched by hand in separate sessions: 68 gates. ST-02 to ST-04 ran with an autonomous orchestrator and subagents: 17, 5 and 4 gates. k_a is dominated by ST-01 and ST-02 (247 of 301 min), so it is likely pessimistic for later stories, while k_m includes first-cycle CI setup.
- **Simulation conventions.** Counting points in the opening evening, and letting an element open alone above the budget, are conventions of the simulation, not rules of the plan. With a budget of 4.8, they set the pace more than N does.
- **Stale state files.** The `state.json` of ST-02, ST-03 and ST-04 were never closed (`current_phase` 7, 7 and 8, no `closed_at`) although their PRs are merged.
