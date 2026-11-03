"""Tests des aides de CI (T-8, T-9, T-11) : matrices US-5 à US-7 de `.ctdd/stories/ST-01/02_specs.md`.

Lancement : `python3 -m unittest scripts/lib/test_ci.py`.
"""

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

import graphe  # noqa: E402
import juge  # noqa: E402
import mutation  # noqa: E402
from mutation import Issue, Rouge, Score  # noqa: E402


def meta(aretes: dict[str, list[str]]) -> dict:
    """`cargo metadata` minimal : {paquet: dépendances directes}. Les `arbitre*` sont membres du workspace."""
    noms = set(aretes) | {d for deps in aretes.values() for d in deps}
    ident = {n: f"{n} 0.1.0 (path+file:///depot/{n})" for n in sorted(noms)}
    return {
        "packages": [{"name": n, "id": ident[n], "manifest_path": f"/depot/{n}/Cargo.toml"} for n in sorted(noms)],
        "workspace_members": [ident[n] for n in sorted(noms) if n.startswith("arbitre")],
        "resolve": {
            "nodes": [
                {
                    "id": ident[n],
                    "deps": [
                        {"name": d.replace("-", "_"), "pkg": ident[d], "dep_kinds": [{"kind": None, "target": None}]}
                        for d in aretes.get(n, [])
                    ],
                }
                for n in sorted(noms)
            ]
        },
    }


ARBRE_CORE_PROPRE = "arbitre-core v0.1.0 (/depot/crates/core) \n"


class TestGraphe(unittest.TestCase):
    """T-8 : matrices US-5 cas 2 à 5, US-6 cas 5 à 7 ; A-6, A-9."""

    def test_us5_cas2_rusqlite_dans_le_coeur(self):
        arbre = ARBRE_CORE_PROPRE + "└── rusqlite v0.41.1 bundled\n"
        lignes = graphe.regle_interdites({"arbitre-core": arbre})
        self.assertEqual(len(lignes), 1)
        self.assertIn("arbitre-core", lignes[0])
        self.assertIn("rusqlite", lignes[0])

    def test_us5_cas3_chrono_clock_transitif(self):
        arbre = ARBRE_CORE_PROPRE + "└── x v1.0.0 \n    └── chrono v0.4.45 clock,std\n"
        self.assertTrue(graphe.regle_interdites({"arbitre-core": arbre}))

    def test_chrono_sans_clock_accepte(self):
        arbre = ARBRE_CORE_PROPRE + "└── chrono v0.4.45 alloc\n"
        self.assertEqual(graphe.regle_interdites({"arbitre-core": arbre}), [])

    def test_us5_cas4_la_cli_peut_lire_l_heure(self):
        self.assertEqual(graphe.regle_interdites({"arbitre-core": ARBRE_CORE_PROPRE}), [])
        self.assertEqual(graphe.regle_aretes(meta({"arbitre": ["arbitre-core", "chrono"], "arbitre-core": []})), [])

    def test_us5_cas5_version_repetee_dans_un_composant(self):
        racine = {"workspace": {"dependencies": {"serde": {"version": "1.0.229"}}}}
        manifestes = {"arbitre-core": {"dependencies": {"serde": "1.0.229"}}}
        lignes = graphe.regle_communes(racine, manifestes)
        self.assertEqual(len(lignes), 1)
        self.assertIn("arbitre-core", lignes[0])
        self.assertIn("serde", lignes[0])

    def test_dependance_de_deux_composants_hors_workspace(self):
        racine = {"workspace": {"dependencies": {}}}
        manifestes = {
            "arbitre-plan": {"dependencies": {"regex": "1"}},
            "arbitre-diff": {"dependencies": {"regex": "1"}},
        }
        self.assertTrue(graphe.regle_communes(racine, manifestes))

    def test_heritage_workspace_accepte(self):
        racine = {"workspace": {"dependencies": {"serde": {"version": "1.0.229"}}}}
        manifestes = {"arbitre-core": {"dev-dependencies": {"serde": {"workspace": True, "features": ["std"]}}}}
        self.assertEqual(graphe.regle_communes(racine, manifestes), [])

    def test_us6_cas5_arete_plan_vers_diff(self):
        lignes = graphe.regle_aretes(meta({"arbitre-plan": ["arbitre-diff"], "arbitre-diff": ["arbitre-core"]}))
        self.assertTrue(any("arbitre-plan" in l and "arbitre-diff" in l for l in lignes))

    def test_us6_cas6_arete_transitive_chaine_nommee(self):
        m = meta({"arbitre-plan": ["x-ext"], "x-ext": ["arbitre-diff"], "arbitre-diff": []})
        lignes = graphe.regle_aretes(m)
        self.assertTrue(any("x-ext" in l and "arbitre-plan" in l and "arbitre-diff" in l for l in lignes))

    def test_us6_cas7_composant_ne_plus_tard(self):
        self.assertEqual(graphe.regle_aretes(meta({"arbitre": ["arbitre-core"], "arbitre-core": []})), [])
        m = meta({"arbitre": ["arbitre-core", "arbitre-diff"], "arbitre-diff": ["arbitre-core"], "arbitre-core": []})
        self.assertEqual(graphe.regle_aretes(m), [])

    def test_calendar_vers_plan_accepte(self):
        m = meta({"arbitre-calendar": ["arbitre-plan"], "arbitre-plan": ["arbitre-core"], "arbitre-core": []})
        self.assertEqual(graphe.regle_aretes(m), [])

    def test_relay_vers_plan_refuse(self):
        m = meta({"arbitre-relay": ["arbitre-plan"], "arbitre-plan": ["arbitre-core"], "arbitre-core": []})
        self.assertTrue(graphe.regle_aretes(m))

    def test_coeur_vers_un_composant_refuse(self):
        self.assertTrue(graphe.regle_aretes(meta({"arbitre-core": ["arbitre-plan"], "arbitre-plan": []})))

    def test_crate_inconnue_rouge(self):
        lignes = graphe.regle_aretes(meta({"arbitre-truc": [], "arbitre-core": []}))
        self.assertTrue(any("arbitre-truc" in l for l in lignes))

    def test_purete_lib_sans_no_std(self):
        sources = {"arbitre-core": {"crates/core/src/lib.rs": "extern crate alloc;\n"}}
        self.assertTrue(graphe.regle_purete(sources))

    def test_purete_extern_crate_std(self):
        sources = {
            "arbitre-core": {
                "crates/core/src/lib.rs": "#![no_std]\nextern crate alloc;\n",
                "crates/core/src/clock.rs": "extern crate std;\n",
            }
        }
        lignes = graphe.regle_purete(sources)
        self.assertTrue(any("clock.rs" in l for l in lignes))

    def test_purete_doc_instant_courant_vert(self):
        sources = {
            "arbitre-core": {
                "crates/core/src/lib.rs": "#![no_std]\nextern crate alloc;\n",
                "crates/core/src/clock.rs": "/// Instant courant\npub trait Clock {}\n",
            }
        }
        self.assertEqual(graphe.regle_purete(sources), [])

    def test_tls_openssl_refuse(self):
        lignes = graphe.regle_tls(meta({"arbitre": ["native-tls"], "native-tls": ["openssl-sys"]}))
        self.assertTrue(any("openssl-sys" in l for l in lignes))
        self.assertTrue(any("native-tls" in l for l in lignes))


ZONES_ST01 = 'Full = ["crates/core/src/hlc.rs", "crates/core/src/merge.rs"]\nStd = ["crates/core/src/clock.rs"]\nGen = ["crates/cli/src/**"]\n'


def issues(fichier: str, caught: int = 0, missed: int = 0, timeout: int = 0, unviable: int = 0) -> list[Issue]:
    sortie = []
    for issue, n in (("caught", caught), ("missed", missed), ("timeout", timeout), ("unviable", unviable)):
        sortie += [Issue(f"{fichier}:{k}:1: {issue} {k}", fichier, issue) for k in range(len(sortie), len(sortie) + n)]
    return sortie


class TestMutation(unittest.TestCase):
    """T-9 : matrice US-7, cas 1 à 8, 10 et 11 ; motifs `full_match`. Le cas 9 relève de `--in-diff` (démonstration)."""

    def test_motif_etoile_ne_traverse_pas_le_separateur(self):
        self.assertFalse(mutation.couvre("crates/*/x.rs", "crates/a/b/x.rs"))
        self.assertTrue(mutation.couvre("crates/*/x.rs", "crates/a/x.rs"))

    def test_motif_double_etoile_zero_segment(self):
        self.assertTrue(mutation.couvre("crates/**/x.rs", "crates/x.rs"))

    def test_motif_double_etoile_n_segments(self):
        self.assertTrue(mutation.couvre("crates/cli/src/**", "crates/cli/src/a/b/main.rs"))

    def test_motif_exact(self):
        self.assertTrue(mutation.couvre("crates/core/src/hlc.rs", "crates/core/src/hlc.rs"))

    def test_motif_sans_correspondance(self):
        self.assertFalse(mutation.couvre("crates/core/src/hlc.rs", "crates/core/src/merge.rs"))

    def zones(self, *textes: str) -> dict[str, list[str]]:
        return mutation.lire_zones({f".ctdd/stories/ST-{i:02}/zones.toml": t for i, t in enumerate(textes)})

    def test_us7_cas1_fichier_non_declare_sous_teste(self):
        s = mutation.scores(issues("crates/core/src/x.rs", caught=6, missed=4), set(), self.zones(ZONES_ST01))
        self.assertEqual(s, [Score("crates/core/src/x.rs", "Full", 6, 10)])
        self.assertEqual(mutation.verdict(s[0]), "rouge")
        texte = mutation.rapport(s)
        for attendu in ("crates/core/src/x.rs", "60", "85"):
            self.assertIn(attendu, texte)

    def test_us7_cas2_std_a_72(self):
        s = mutation.scores(issues("crates/core/src/clock.rs", caught=18, missed=7), set(), self.zones(ZONES_ST01))
        self.assertEqual(s[0].zone, "Std")
        self.assertEqual(mutation.verdict(s[0]), "vert")

    def test_us7_cas3_std_et_full_le_plus_exigeant(self):
        zones = self.zones('Std = ["crates/core/src/*.rs"]\n', 'Full = ["crates/core/src/a.rs"]\n')
        self.assertEqual(mutation.zone_de("crates/core/src/a.rs", zones), "Full")
        s = mutation.scores(issues("crates/core/src/a.rs", caught=8, missed=2), set(), zones)
        self.assertEqual(mutation.verdict(s[0]), "rouge")

    def test_us7_cas4_aucun_zones_toml(self):
        self.assertEqual(mutation.zone_de("crates/core/src/a.rs", mutation.lire_zones({})), "Full")

    def test_us7_cas5_rien_a_mesurer(self):
        self.assertEqual(mutation.scores([], set(), self.zones(ZONES_ST01)), [])

    def test_us7_cas6_score_egal_au_seuil(self):
        s = mutation.scores(issues("crates/core/src/hlc.rs", caught=17, missed=3), set(), self.zones(ZONES_ST01))
        self.assertEqual(mutation.verdict(s[0]), "vert")

    def test_us7_cas7_equivalent_justifie(self):
        liste = issues("crates/core/src/hlc.rs", caught=17, missed=3)
        texte = f'[[equivalent]]\nmutant = "{liste[-1].nom}"\njustification = "aucun effet observable"\n'
        equivalents = mutation.lire_equivalents({".ctdd/stories/ST-01/mutants_equivalents.toml": texte})
        s = mutation.scores(liste, equivalents, self.zones(ZONES_ST01))
        self.assertEqual((s[0].detectes, s[0].comptes), (17, 19))
        self.assertEqual(mutation.verdict(s[0]), "vert")

    def test_us7_cas8_equivalent_sans_justification(self):
        for texte in ('[[equivalent]]\nmutant = "a.rs:1:1: x"\njustification = ""\n', '[[equivalent]]\nmutant = "a.rs:1:1: x"\n'):
            with self.assertRaises(Rouge):
                mutation.lire_equivalents({".ctdd/stories/ST-01/mutants_equivalents.toml": texte})

    def test_us7_cas10_zones_mal_forme(self):
        with self.assertRaisesRegex(Rouge, "ST-07/zones.toml"):
            mutation.lire_zones({".ctdd/stories/ST-07/zones.toml": "Full = [\n"})

    def test_zone_inconnue_rouge(self):
        with self.assertRaisesRegex(Rouge, "ST-07/zones.toml"):
            mutation.lire_zones({".ctdd/stories/ST-07/zones.toml": 'Moyen = ["a.rs"]\n'})

    def test_us7_cas11_tous_non_construisibles(self):
        s = mutation.scores(issues("crates/core/src/hlc.rs", unviable=4), set(), self.zones(ZONES_ST01))
        self.assertEqual(mutation.verdict(s[0]), "neutre")

    def test_timeout_compte_comme_detecte(self):
        s = mutation.scores(issues("crates/core/src/hlc.rs", timeout=1), set(), self.zones(ZONES_ST01))
        self.assertEqual((s[0].detectes, s[0].comptes), (1, 1))

    def test_gen_informatif(self):
        s = mutation.scores(issues("crates/cli/src/main.rs", missed=5), set(), self.zones(ZONES_ST01))
        self.assertEqual(mutation.verdict(s[0]), "info")

    def test_gen_interdit_dans_le_coeur(self):
        zones = self.zones('Gen = ["crates/core/src/**"]\n')
        self.assertEqual(mutation.garde_gen(["crates/core/src/hlc.rs"], zones), ["crates/core/src/hlc.rs"])

    def test_sortie_illisible_rouge(self):
        with self.assertRaises(Rouge):
            mutation.lire_issues({"inattendu": True})

    def test_rapport_trie_par_fichier(self):
        texte = mutation.rapport([Score("b.rs", "Full", 1, 1), Score("a.rs", "Full", 1, 1)])
        self.assertLess(texte.index("a.rs"), texte.index("b.rs"))


ZONES_BASE = {".ctdd/stories/ST-01/zones.toml": ZONES_ST01}
RS_BASE = ["crates/core/src/lib.rs", "crates/core/src/merge.rs", "crates/core/src/clock.rs", "crates/cli/src/main.rs"]


def zones_avec(**ajouts: str) -> dict[str, str]:
    return {**ZONES_BASE, **{f".ctdd/stories/{k.replace('_', '-')}/zones.toml": v for k, v in ajouts.items()}}


class TestJuge(unittest.TestCase):
    """T-11 : règles d'A-8 sur `juger`, données en mémoire."""

    def juger(self, changes, zones_tete=None, labels=frozenset()):
        return juge.juger(changes, ZONES_BASE, ZONES_BASE if zones_tete is None else zones_tete, RS_BASE, set(labels))

    def test_chemins_proteges_rouges(self):
        for chemin in ("scripts/lib/mutation.py", "scripts/ci.d/45-st05-prep.sh", ".github/workflows/ci.yml"):
            rouge, lignes = self.juger([chemin])
            self.assertTrue(rouge, chemin)
            self.assertTrue(any(chemin in l for l in lignes))

    def test_ctdd_et_crates_seuls_verts(self):
        self.assertFalse(self.juger([".ctdd/stories/ST-02/02_specs.md", "crates/core/src/merge.rs"])[0])

    def test_full_vers_std_rouge(self):
        declarations = 'Full = ["crates/core/src/hlc.rs"]\nStd = ["crates/core/src/clock.rs", "crates/core/src/merge.rs"]\n'
        tete = {".ctdd/stories/ST-01/zones.toml": declarations}
        rouge, lignes = self.juger([".ctdd/stories/ST-01/zones.toml"], tete)
        self.assertTrue(rouge)
        self.assertTrue(any("merge.rs" in l for l in lignes))

    def test_motif_std_large_rouge(self):
        rouge, lignes = self.juger([".ctdd/stories/ST-05/zones.toml"], zones_avec(ST_05='Std = ["crates/core/src/**"]\n'))
        self.assertTrue(rouge)
        self.assertTrue(any("crates/core/src/lib.rs" in l for l in lignes))

    def test_motif_sur_fichier_absent_de_la_base_vert(self):
        self.assertFalse(self.juger(["crates/core/src/neuf.rs"], zones_avec(ST_05='Std = ["crates/core/src/neuf.rs"]\n'))[0])

    def test_std_vers_full_vert(self):
        self.assertFalse(self.juger([], zones_avec(ST_05='Full = ["crates/core/src/clock.rs"]\n'))[0])

    def test_zones_de_la_tete_illisible_rouge(self):
        rouge, lignes = self.juger([], zones_avec(ST_05="Full = [\n"))
        self.assertTrue(rouge)
        self.assertTrue(any("ST-05/zones.toml" in l for l in lignes))

    def test_label_et_proteges_seuls_vert_liste_affichee(self):
        rouge, lignes = self.juger(["scripts/lib/mutation.py"], labels={"ci-change"})
        self.assertFalse(rouge)
        self.assertTrue(any("scripts/lib/mutation.py" in l for l in lignes))

    def test_label_et_crates_rouge(self):
        self.assertTrue(self.juger(["scripts/lib/mutation.py", "crates/core/src/hlc.rs"], labels={"ci-change"})[0])


if __name__ == "__main__":
    unittest.main()
