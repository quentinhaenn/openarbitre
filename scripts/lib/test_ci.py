"""Tests of the CI helpers (T-8, T-9, T-11): US-5 to US-7 matrices of `.ctdd/stories/ST-01/02_specs.md`.

Run with: `python3 -m unittest scripts/lib/test_ci.py`.
"""

import json
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

import graph  # noqa: E402
import judge  # noqa: E402
import mutation  # noqa: E402
from mutation import Outcome, RedError, Score  # noqa: E402


def meta(edges: dict[str, list[str]]) -> dict:
    """Minimal `cargo metadata`: {package: direct dependencies}. The `arbitre*` packages are workspace members."""
    names = set(edges) | {d for deps in edges.values() for d in deps}
    ids = {n: f"{n} 0.1.0 (path+file:///repo/{n})" for n in sorted(names)}
    return {
        "packages": [{"name": n, "id": ids[n], "manifest_path": f"/repo/{n}/Cargo.toml"} for n in sorted(names)],
        "workspace_members": [ids[n] for n in sorted(names) if n.startswith("arbitre")],
        "resolve": {
            "nodes": [
                {
                    "id": ids[n],
                    "deps": [
                        {"name": d.replace("-", "_"), "pkg": ids[d], "dep_kinds": [{"kind": None, "target": None}]}
                        for d in edges.get(n, [])
                    ],
                }
                for n in sorted(names)
            ]
        },
    }


CLEAN_CORE_TREE = "arbitre-core v0.1.0 (/repo/crates/core) \n"


class TestGraph(unittest.TestCase):
    """T-8: matrices US-5 cases 2 to 5, US-6 cases 5 to 7; A-6, A-9."""

    def test_us5_case2_rusqlite_in_the_core(self):
        tree = CLEAN_CORE_TREE + "└── rusqlite v0.41.1 bundled\n"
        lines = graph.rule_forbidden({"arbitre-core": tree})
        self.assertEqual(len(lines), 1)
        self.assertIn("arbitre-core", lines[0])
        self.assertIn("rusqlite", lines[0])

    def test_us5_case3_chrono_clock_transitive(self):
        tree = CLEAN_CORE_TREE + "└── x v1.0.0 \n    └── chrono v0.4.45 clock,std\n"
        self.assertTrue(graph.rule_forbidden({"arbitre-core": tree}))

    def test_chrono_without_clock_accepted(self):
        tree = CLEAN_CORE_TREE + "└── chrono v0.4.45 alloc\n"
        self.assertEqual(graph.rule_forbidden({"arbitre-core": tree}), [])

    def test_us5_case4_cli_may_read_the_time(self):
        self.assertEqual(graph.rule_forbidden({"arbitre-core": CLEAN_CORE_TREE}), [])
        self.assertEqual(graph.rule_edges(meta({"arbitre": ["arbitre-core", "chrono"], "arbitre-core": []})), [])

    def test_us5_case5_version_repeated_in_a_component(self):
        root = {"workspace": {"dependencies": {"serde": {"version": "1.0.229"}}}}
        manifests = {"arbitre-core": {"dependencies": {"serde": "1.0.229"}}}
        lines = graph.rule_shared(root, manifests)
        self.assertEqual(len(lines), 1)
        self.assertIn("arbitre-core", lines[0])
        self.assertIn("serde", lines[0])

    def test_dependency_of_two_components_outside_workspace(self):
        root = {"workspace": {"dependencies": {}}}
        manifests = {
            "arbitre-plan": {"dependencies": {"regex": "1"}},
            "arbitre-diff": {"dependencies": {"regex": "1"}},
        }
        self.assertTrue(graph.rule_shared(root, manifests))

    def test_workspace_inheritance_accepted(self):
        root = {"workspace": {"dependencies": {"serde": {"version": "1.0.229"}}}}
        manifests = {"arbitre-core": {"dev-dependencies": {"serde": {"workspace": True, "features": ["std"]}}}}
        self.assertEqual(graph.rule_shared(root, manifests), [])

    def test_us6_case5_edge_plan_to_diff(self):
        lines = graph.rule_edges(meta({"arbitre-plan": ["arbitre-diff"], "arbitre-diff": ["arbitre-core"]}))
        self.assertTrue(any("arbitre-plan" in l and "arbitre-diff" in l for l in lines))

    def test_us6_case6_transitive_edge_chain_named(self):
        m = meta({"arbitre-plan": ["x-ext"], "x-ext": ["arbitre-diff"], "arbitre-diff": []})
        lines = graph.rule_edges(m)
        self.assertTrue(any("x-ext" in l and "arbitre-plan" in l and "arbitre-diff" in l for l in lines))

    def test_us6_case7_component_not_later(self):
        self.assertEqual(graph.rule_edges(meta({"arbitre": ["arbitre-core"], "arbitre-core": []})), [])
        m = meta({"arbitre": ["arbitre-core", "arbitre-diff"], "arbitre-diff": ["arbitre-core"], "arbitre-core": []})
        self.assertEqual(graph.rule_edges(m), [])

    def test_calendar_to_plan_accepted(self):
        m = meta({"arbitre-calendar": ["arbitre-plan"], "arbitre-plan": ["arbitre-core"], "arbitre-core": []})
        self.assertEqual(graph.rule_edges(m), [])

    def test_relay_to_plan_rejected(self):
        m = meta({"arbitre-relay": ["arbitre-plan"], "arbitre-plan": ["arbitre-core"], "arbitre-core": []})
        self.assertTrue(graph.rule_edges(m))

    def test_core_to_a_component_rejected(self):
        self.assertTrue(graph.rule_edges(meta({"arbitre-core": ["arbitre-plan"], "arbitre-plan": []})))

    def test_unknown_crate_red(self):
        lines = graph.rule_edges(meta({"arbitre-thing": [], "arbitre-core": []}))
        self.assertTrue(any("arbitre-thing" in l for l in lines))

    def test_purity_lib_without_no_std(self):
        sources = {"arbitre-core": {"crates/core/src/lib.rs": "extern crate alloc;\n"}}
        self.assertTrue(graph.rule_purity(sources))

    def test_purity_extern_crate_std(self):
        sources = {
            "arbitre-core": {
                "crates/core/src/lib.rs": "#![no_std]\nextern crate alloc;\n",
                "crates/core/src/clock.rs": "extern crate std;\n",
            }
        }
        lines = graph.rule_purity(sources)
        self.assertTrue(any("clock.rs" in l for l in lines))

    def test_purity_doc_current_instant_green(self):
        sources = {
            "arbitre-core": {
                "crates/core/src/lib.rs": "#![no_std]\nextern crate alloc;\n",
                "crates/core/src/clock.rs": "/// Current instant\npub trait Clock {}\n",
            }
        }
        self.assertEqual(graph.rule_purity(sources), [])

    def test_tls_openssl_rejected(self):
        lines = graph.rule_tls(meta({"arbitre": ["native-tls"], "native-tls": ["openssl-sys"]}))
        self.assertTrue(any("openssl-sys" in l for l in lines))
        self.assertTrue(any("native-tls" in l for l in lines))


ZONES_ST01 = 'Full = ["crates/core/src/hlc.rs", "crates/core/src/merge.rs"]\nStd = ["crates/core/src/clock.rs"]\nGen = ["crates/cli/src/**"]\n'


def outcomes(file: str, caught: int = 0, missed: int = 0, timeout: int = 0, unviable: int = 0) -> list[Outcome]:
    out = []
    for result, n in (("caught", caught), ("missed", missed), ("timeout", timeout), ("unviable", unviable)):
        out += [Outcome(f"{file}:{k}:1: {result} {k}", file, result) for k in range(len(out), len(out) + n)]
    return out


class TestMutation(unittest.TestCase):
    """T-9: US-7 matrix, cases 1 to 8, 10 and 11; `full_match` patterns. Case 9 belongs to `--in-diff` (demonstration)."""

    def test_pattern_star_does_not_cross_separator(self):
        self.assertFalse(mutation.covers("crates/*/x.rs", "crates/a/b/x.rs"))
        self.assertTrue(mutation.covers("crates/*/x.rs", "crates/a/x.rs"))

    def test_pattern_double_star_zero_segments(self):
        self.assertTrue(mutation.covers("crates/**/x.rs", "crates/x.rs"))

    def test_pattern_double_star_n_segments(self):
        self.assertTrue(mutation.covers("crates/cli/src/**", "crates/cli/src/a/b/main.rs"))

    def test_pattern_exact(self):
        self.assertTrue(mutation.covers("crates/core/src/hlc.rs", "crates/core/src/hlc.rs"))

    def test_pattern_no_match(self):
        self.assertFalse(mutation.covers("crates/core/src/hlc.rs", "crates/core/src/merge.rs"))

    def zones(self, *texts: str) -> dict[str, list[str]]:
        return mutation.read_zones({f".ctdd/stories/ST-{i:02}/zones.toml": t for i, t in enumerate(texts)})

    def test_us7_case1_undeclared_file_undertested(self):
        s = mutation.scores(outcomes("crates/core/src/x.rs", caught=6, missed=4), set(), self.zones(ZONES_ST01))
        self.assertEqual(s, [Score("crates/core/src/x.rs", "Full", 6, 10)])
        self.assertEqual(mutation.verdict(s[0]), "red")
        text = mutation.report(s)
        for expected in ("crates/core/src/x.rs", "60", "85"):
            self.assertIn(expected, text)

    def test_us7_case2_std_at_72(self):
        s = mutation.scores(outcomes("crates/core/src/clock.rs", caught=18, missed=7), set(), self.zones(ZONES_ST01))
        self.assertEqual(s[0].zone, "Std")
        self.assertEqual(mutation.verdict(s[0]), "green")

    def test_us7_case3_std_and_full_strictest_wins(self):
        zones = self.zones('Std = ["crates/core/src/*.rs"]\n', 'Full = ["crates/core/src/a.rs"]\n')
        self.assertEqual(mutation.zone_of("crates/core/src/a.rs", zones), "Full")
        s = mutation.scores(outcomes("crates/core/src/a.rs", caught=8, missed=2), set(), zones)
        self.assertEqual(mutation.verdict(s[0]), "red")

    def test_us7_case4_no_zones_toml(self):
        self.assertEqual(mutation.zone_of("crates/core/src/a.rs", mutation.read_zones({})), "Full")

    def test_us7_case5_nothing_to_measure(self):
        self.assertEqual(mutation.scores([], set(), self.zones(ZONES_ST01)), [])

    def test_us7_case6_score_equal_to_threshold(self):
        s = mutation.scores(outcomes("crates/core/src/hlc.rs", caught=17, missed=3), set(), self.zones(ZONES_ST01))
        self.assertEqual(mutation.verdict(s[0]), "green")

    def test_us7_case7_justified_equivalent(self):
        items = outcomes("crates/core/src/hlc.rs", caught=17, missed=3)
        text = f'[[equivalent]]\nmutant = "{items[-1].name}"\njustification = "no observable effect"\n'
        equivalents = mutation.read_equivalents({".ctdd/stories/ST-01/mutants_equivalents.toml": text})
        s = mutation.scores(items, equivalents, self.zones(ZONES_ST01))
        self.assertEqual((s[0].detected, s[0].counted), (17, 19))
        self.assertEqual(mutation.verdict(s[0]), "green")

    def test_us7_case8_equivalent_without_justification(self):
        for text in ('[[equivalent]]\nmutant = "a.rs:1:1: x"\njustification = ""\n', '[[equivalent]]\nmutant = "a.rs:1:1: x"\n'):
            with self.assertRaises(RedError):
                mutation.read_equivalents({".ctdd/stories/ST-01/mutants_equivalents.toml": text})

    def test_us7_case10_zones_malformed(self):
        with self.assertRaisesRegex(RedError, "ST-07/zones.toml"):
            mutation.read_zones({".ctdd/stories/ST-07/zones.toml": "Full = [\n"})

    def test_unknown_zone_red(self):
        with self.assertRaisesRegex(RedError, "ST-07/zones.toml"):
            mutation.read_zones({".ctdd/stories/ST-07/zones.toml": 'Medium = ["a.rs"]\n'})

    def test_us7_case11_all_unviable(self):
        s = mutation.scores(outcomes("crates/core/src/hlc.rs", unviable=4), set(), self.zones(ZONES_ST01))
        self.assertEqual(mutation.verdict(s[0]), "neutral")

    def test_timeout_counts_as_detected(self):
        s = mutation.scores(outcomes("crates/core/src/hlc.rs", timeout=1), set(), self.zones(ZONES_ST01))
        self.assertEqual((s[0].detected, s[0].counted), (1, 1))

    def test_gen_informational(self):
        s = mutation.scores(outcomes("crates/cli/src/main.rs", missed=5), set(), self.zones(ZONES_ST01))
        self.assertEqual(mutation.verdict(s[0]), "info")

    def test_gen_forbidden_in_the_core(self):
        zones = self.zones('Gen = ["crates/core/src/**"]\n')
        self.assertEqual(mutation.gen_guard(["crates/core/src/hlc.rs"], zones), ["crates/core/src/hlc.rs"])

    def test_unreadable_output_red(self):
        with self.assertRaises(RedError):
            mutation.read_outcomes({"unexpected": True})

    def test_report_sorted_by_file(self):
        text = mutation.report([Score("b.rs", "Full", 1, 1), Score("a.rs", "Full", 1, 1)])
        self.assertLess(text.index("a.rs"), text.index("b.rs"))


FIXTURES = Path(__file__).parent / "fixtures" / "mutants"


class TestRealFormat(unittest.TestCase):
    """T-9: real `outcomes.json` of cargo-mutants 27.1.0 (captured on a throwaway crate, R-3)."""

    def load(self, name: str) -> dict:
        return json.loads((FIXTURES / name).read_text())

    def test_real_outcomes(self):
        items = mutation.read_outcomes(self.load("outcomes_mixed.json"))
        self.assertEqual([i.result for i in items], ["missed", "missed", "unviable", "unviable", "timeout"])
        self.assertEqual(items[0], Outcome("src/lib.rs:2:32: replace bound -> bool with true", "src/lib.rs", "missed"))

    def test_real_red_baseline(self):
        with self.assertRaises(RedError):
            mutation.read_outcomes(self.load("outcomes_base_red.json"))


ZONES_BASE = {".ctdd/stories/ST-01/zones.toml": ZONES_ST01}
RS_BASE = ["crates/core/src/lib.rs", "crates/core/src/merge.rs", "crates/core/src/clock.rs", "crates/cli/src/main.rs"]


def zones_with(**additions: str) -> dict[str, str]:
    return {**ZONES_BASE, **{f".ctdd/stories/{k.replace('_', '-')}/zones.toml": v for k, v in additions.items()}}


class TestJudge(unittest.TestCase):
    """T-11: A-8 rules on `judge`, in-memory data."""

    def judge(self, changes, zones_head=None, labels=frozenset()):
        return judge.judge(changes, ZONES_BASE, ZONES_BASE if zones_head is None else zones_head, RS_BASE, set(labels))

    def test_protected_paths_red(self):
        for path in ("scripts/lib/mutation.py", "scripts/ci.d/45-st05-prep.sh", ".github/workflows/ci.yml", ".gitleaksignore", ".cargo/config.toml"):
            rouge, lines = self.judge([path])
            self.assertTrue(rouge, path)
            self.assertTrue(any(path in l for l in lines))

    def test_ctdd_and_crates_alone_green(self):
        self.assertFalse(self.judge([".ctdd/stories/ST-02/02_specs.md", "crates/core/src/merge.rs"])[0])

    def test_full_to_std_red(self):
        declarations = 'Full = ["crates/core/src/hlc.rs"]\nStd = ["crates/core/src/clock.rs", "crates/core/src/merge.rs"]\n'
        head = {".ctdd/stories/ST-01/zones.toml": declarations}
        rouge, lines = self.judge([".ctdd/stories/ST-01/zones.toml"], head)
        self.assertTrue(rouge)
        self.assertTrue(any("merge.rs" in l for l in lines))

    def test_broad_std_pattern_red(self):
        rouge, lines = self.judge([".ctdd/stories/ST-05/zones.toml"], zones_with(ST_05='Std = ["crates/core/src/**"]\n'))
        self.assertTrue(rouge)
        self.assertTrue(any("crates/core/src/lib.rs" in l for l in lines))

    def test_pattern_on_file_absent_from_base_green(self):
        self.assertFalse(self.judge(["crates/core/src/new.rs"], zones_with(ST_05='Std = ["crates/core/src/new.rs"]\n'))[0])

    def test_std_to_full_green(self):
        self.assertFalse(self.judge([], zones_with(ST_05='Full = ["crates/core/src/clock.rs"]\n'))[0])

    def test_unreadable_head_zones_red(self):
        rouge, lines = self.judge([], zones_with(ST_05="Full = [\n"))
        self.assertTrue(rouge)
        self.assertTrue(any("ST-05/zones.toml" in l for l in lines))

    def test_label_and_protected_only_green_list_shown(self):
        rouge, lines = self.judge(["scripts/lib/mutation.py"], labels={"ci-change"})
        self.assertFalse(rouge)
        self.assertTrue(any("scripts/lib/mutation.py" in l for l in lines))

    def test_label_and_crates_red(self):
        self.assertTrue(self.judge(["scripts/lib/mutation.py", "crates/core/src/hlc.rs"], labels={"ci-change"})[0])


if __name__ == "__main__":
    unittest.main()
