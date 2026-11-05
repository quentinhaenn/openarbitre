# Market watch: Linear (2026-11-04)

- Diffs (all plans, 2026-05-27): review/merge GitHub PRs inside Linear; Reviews tab ordered "closest to shipping"; shows stacks; optional external risk score 1–4. https://linear.app/docs/diffs
- Guided reviews (GA 2026-07-30): PR split into sections with "what changed and why" (likely LLM). No rule/invariant check. https://linear.app/now/reviewing-code-in-the-agent-era
- Triage rules / Issue SLAs (Business+): if/then rules and priority deadlines on issues, not PRs. https://linear.app/docs/triage
- Releases (2026-04-30): CI/CD pipelines → which issues reached which environment.

Decision diff vs Linear: Linear explains, never gives a deterministic verdict; no invariants file, no CI gate, no per-decision accept/reject, no iteration diff; SaaS + GitHub only.
BL-1 vs Linear: opaque review ordering, no merge-dependency graph beyond stacks, no unblock/zone factors, no adjustable weights or breakdown.

Implications:
1. Integrations: push decision-diff verdict into Diffs' risk-score slot (third-party write UNVERIFIED); BL-1 can read Linear priority/SLA as "customer urgency".
2. Positioning: "Linear explains a PR; OpenArbitre decides whether it respects your architecture — deterministically, in your CI, from a file you version."
3. Do not build: AI story-style PR summaries, in-tool diff viewer/merge button, issue SLAs/triage rules, release tracking. Keep AI-proposed decisions subordinate to deterministic invariants.
