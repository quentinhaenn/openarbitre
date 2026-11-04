# Backlog — stories not yet placed in the plan

Placed into `04_plan.md` at the recalibration after ST-04. Each entry is a raw need, not a spec.

## BL-1: PR dependency graph, sorted by measured priority

**Raised by Quentin, 2026-11-04.**

**Need.** A view of the PRs I have to handle (review, merge, follow up), shown as a graph: edges are merge dependencies, and nodes are ordered by a measured priority, not a hand-set one.

**Dependencies (edges).**
- Within one repository: stacked PRs (a PR based on another PR's branch), explicit references (`Depends on #12`). → **OSS**.
- Across repositories and services of an organisation: merge dependencies between PRs of different repos. → **Enterprise** (outside the public repository; `CADRAGE.md`: "multi-repository graph").

**Priority factors.**
1. Customer urgency — an external signal (ticket, label, named customer), fed by a source adapter, like the calendar for planning.
2. What it unblocks — number of PRs waiting behind it in the graph.
3. Age.
4. Criticality of the touched zone (`Full` > `Std` > `Gen`, from the `zones.toml` files; provided by the decision diff module).

**Constraint: the score stays explainable.** The tool arbitrates nothing alone (planning principle):
- weighted sum with user-adjustable weights;
- per-PR breakdown shown (e.g. "customer urgency +40, unblocks 3 PRs +25, 9 days +10, zone Full +15");
- when two PRs are too close to order, a question is raised to the user instead of an arbitrary order.

**Placement.** Mainly the planning module: PRs become items of the sorted stack, the graph is a third view next to the stack and the calendar, enabled by a generic "blocks" relation between items. Decision diff supplies zone criticality. Two modules, so the 10-interview guard does not apply.

**Open at placement time.** Source of customer urgency (which tracker first), default weights, the threshold for "too close", the forge API used to read PRs (GitHub first).
