//! Appareils simulés, oracle causal, convergence (T-7). PROP-ST01-2, INV-5, INV-ST01-1, INV-ST01-2.
//! Matrices US-1 cas 4, 5, 6, 9 ; US-2 cas 5 ; US-3 cas 8 (scénarios générés avec suppressions).

mod support;

use std::cell::RefCell;
use std::collections::BTreeSet;

use arbitre_core::*;
use proptest::prelude::*;
use proptest::test_runner::TestRunner;
use support::*;

fn r_de(x: &Journal) -> (Projection, Vec<Ecriture>, Vec<Conflit>) {
    (x.projection(), x.ecartees(), x.conflits())
}

fn recu(lots: &[&[Ecriture]]) -> Journal {
    let mut x = Journal::nouveau(app(100));
    for lot in lots {
        x.fusionner(lot).unwrap();
    }
    x
}

/// Toutes les écritures d'un scénario livré en entier, et un masque pour les répartir en lots.
fn s_et_masque() -> impl Strategy<Value = (Vec<Ecriture>, Vec<bool>, Vec<bool>)> {
    scenario().prop_flat_map(|sc| {
        let mut sim = Simulation::nouvelle(&sc);
        for action in &sc.actions {
            sim.appliquer(action);
        }
        sim.tout_livrer();
        let s = sim.appareils[0].journal.ecritures();
        let n = s.len();
        (Just(s), prop::collection::vec(any::<bool>(), n), prop::collection::vec(any::<bool>(), n))
    })
}

fn partager(s: &[Ecriture], masque: &[bool]) -> (Vec<Ecriture>, Vec<Ecriture>) {
    let (a, b): (Vec<_>, Vec<_>) = s.iter().cloned().zip(masque).partition(|(_, m)| **m);
    (a.into_iter().map(|(w, _)| w).collect(), b.into_iter().map(|(w, _)| w).collect())
}

proptest! {
    #![proptest_config(config())]

    /// INV-ST01-2, F-2, D-10 : après chaque action, sur chaque appareil, `conflits()` = oracle ;
    /// la concaténation des `nouveaux_conflits` est sans doublon et égale `conflits()` en ensemble.
    #[test]
    fn concurrentes_ssi_conflit(sc in scenario()) {
        let mut sim = Simulation::nouvelle(&sc);
        let mut cumul: Vec<Vec<Conflit>> = vec![Vec::new(); sim.appareils.len()];
        let mut verifier = |sim: &Simulation, rapports: Vec<(usize, Rapport)>| -> Result<(), TestCaseError> {
            for (i, r) in rapports {
                cumul[i].extend(r.nouveaux_conflits);
            }
            let registre = sim.registre();
            for (i, appareil) in sim.appareils.iter().enumerate() {
                let conflits = appareil.journal.conflits();
                prop_assert_eq!(&conflits, &attendus(&appareil.journal.ecritures(), &registre));
                let ensemble: BTreeSet<Conflit> = cumul[i].iter().cloned().collect();
                prop_assert_eq!(ensemble.len(), cumul[i].len());
                prop_assert_eq!(ensemble, conflits.into_iter().collect::<BTreeSet<_>>());
            }
            Ok(())
        };
        for action in &sc.actions {
            let rapports = sim.appliquer(action);
            verifier(&sim, rapports)?;
        }
        let rapports = sim.tout_livrer();
        verifier(&sim, rapports)?;
    }

    /// PROP-ST01-2, US-1 cas 4 et 5, US-2 cas 5 : en fin de scénario, `R(S)` identique partout.
    #[test]
    fn convergence(sc in scenario()) {
        let mut sim = Simulation::nouvelle(&sc);
        for action in &sc.actions {
            sim.appliquer(action);
        }
        sim.tout_livrer();
        let reference = r_de(&sim.appareils[0].journal);
        for appareil in &sim.appareils[1..] {
            prop_assert_eq!(&r_de(&appareil.journal), &reference);
        }
    }

    /// INV-5 : commutativité, associativité, idempotence, neutre (US-1 cas 6, US-3 cas 8, AC-3.5).
    #[test]
    fn inv_5_algebre((s, m1, m2) in s_et_masque()) {
        let (a, reste) = partager(&s, &m1);
        let (b, c) = partager(&reste, &m2[..reste.len()]);
        let ab_c = r_de(&recu(&[&a, &b, &c]));
        prop_assert_eq!(&r_de(&recu(&[&b, &a, &c])), &ab_c);
        prop_assert_eq!(&r_de(&recu(&[&c, &b, &a])), &ab_c);
        let bc: Vec<Ecriture> = b.iter().chain(&c).cloned().collect();
        prop_assert_eq!(&r_de(&recu(&[&a, &bc])), &ab_c);
        let mut x = recu(&[&a, &b, &c]);
        prop_assert_eq!(x.fusionner(&a).unwrap(), Rapport::default());
        prop_assert_eq!(x.fusionner(&[]).unwrap(), Rapport::default());
        prop_assert_eq!(&r_de(&x), &ab_c);
    }

    /// INV-ST01-1, US-1 cas 9 : sorties identiques sous permutation de l'ordre interne d'un lot.
    #[test]
    fn determinisme_permutation((s, _, _) in s_et_masque(), graine in any::<u64>()) {
        let mut permute = s.clone();
        let mut r = graine;
        for i in (1..permute.len()).rev() {
            r = r.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            permute.swap(i, (r >> 33) as usize % (i + 1));
        }
        let (mut x, mut y) = (Journal::nouveau(app(100)), Journal::nouveau(app(100)));
        prop_assert_eq!(x.fusionner(&s).unwrap(), y.fusionner(&permute).unwrap());
        prop_assert_eq!(r_de(&x), r_de(&y));
    }
}

/// R-7 : sur les 1 000 scénarios de la graine, chaque famille d'E-1.1 apparaît au moins une fois.
#[test]
fn couverture_des_familles() {
    let vues = RefCell::new(BTreeSet::new());
    TestRunner::new(config())
        .run(&scenario(), |sc| {
            let mut sim = Simulation::nouvelle(&sc);
            for action in &sc.actions {
                sim.appliquer(action);
            }
            sim.tout_livrer();
            vues.borrow_mut().extend(sim.familles());
            Ok(())
        })
        .unwrap();
    let toutes = BTreeSet::from([
        Famille::LotVide,
        Famille::Doublon,
        Famille::TroisAppareils,
        Famille::MemeL,
        Famille::MemeLCDeuxAppareils,
        Famille::Suppression,
        Famille::ConcurrentesMemeValeur,
        Famille::EcrasementSequentiel,
        Famille::ContreExempleF1,
    ]);
    assert_eq!(vues.into_inner(), toutes);
}
