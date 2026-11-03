//! Marques, horloge hybride, avances, horloge simulée (T-3). Matrice US-4, cas 1 à 10.

mod support;

use std::num::NonZeroU64;

use arbitre_core::*;
use proptest::prelude::*;
use support::*;

const MIN: u64 = 60_000;
const HEURE: u64 = 3_600_000;

fn seuil(ms: u64) -> Seuil {
    Seuil(NonZeroU64::new(ms).unwrap())
}

fn ecr_l(l: u64, a: u128) -> Ecriture {
    ecriture(donnee("d"), champ("f", "v"), marque(l, 0, a))
}

#[test]
fn us4_cas1_correction_sur_appareil_en_retard() {
    let t0 = 1_000_000;
    let source_ordi = HorlogeSimulee::nouvelle(t0 + 5 * MIN);
    let source_tel = HorlogeSimulee::nouvelle(t0);
    let mut ordi = Horloge::nouvelle(app(1));
    let mut tel = Horloge::nouvelle(app(2));
    let m = ordi.emettre(source_ordi.now()).unwrap();
    tel.recevoir(&m);
    source_tel.avancer(MIN);
    let m2 = tel.emettre(source_tel.now()).unwrap();
    assert!(m2 > m);
}

#[test]
fn us4_cas2_deux_emissions_meme_instant() {
    let mut h = Horloge::nouvelle(app(1));
    let m1 = h.emettre(100).unwrap();
    let m2 = h.emettre(100).unwrap();
    assert!(m2 > m1);
    assert_eq!((m2.l, m2.c), (100, m1.c + 1));
}

#[test]
fn us4_cas3_cent_mille_emissions_meme_instant() {
    let mut h = Horloge::nouvelle(app(1));
    let mut precedente = h.emettre(100).unwrap();
    for _ in 1..100_000 {
        let m = h.emettre(100).unwrap();
        assert!(m > precedente);
        precedente = m;
    }
}

#[test]
fn us4_cas4_heure_qui_recule() {
    let mut h = Horloge::nouvelle(app(1));
    let m1 = h.emettre(100).unwrap();
    let m2 = h.emettre(40).unwrap();
    assert!(m2 > m1);
    assert_eq!(m2.l, 100);
}

#[test]
fn us4_cas5_heure_qui_avance_apres_gel() {
    let mut h = Horloge::nouvelle(app(1));
    for _ in 0..8 {
        h.emettre(100).unwrap();
    }
    assert_eq!(h.emettre(150).unwrap(), marque(150, 0, 1));
}

#[test]
fn us4_cas7_avance_de_trois_heures_signalee_et_adoptee() {
    let maintenant = 10 * HEURE;
    let lot = [ecr_l(maintenant + 3 * HEURE, 7)];
    assert_eq!(
        avances(&lot, maintenant, seuil(HEURE)),
        vec![Avance {
            appareil: app(7),
            avance_ms: 3 * HEURE
        }]
    );
    let mut h = Horloge::nouvelle(app(1));
    h.recevoir(&lot[0].marque);
    assert_eq!(h.emettre(maintenant).unwrap().l, maintenant + 3 * HEURE);
}

#[test]
fn us4_cas8_avance_egale_au_seuil_non_signalee() {
    let lot = [ecr_l(1_000 + HEURE, 7)];
    assert!(avances(&lot, 1_000, SEUIL_DEFAUT).is_empty());
}

#[test]
fn us4_cas9_seuil_regle_a_dix_minutes() {
    let lot = [ecr_l(1_000 + 20 * MIN, 7)];
    assert_eq!(
        avances(&lot, 1_000, seuil(10 * MIN)),
        vec![Avance {
            appareil: app(7),
            avance_ms: 20 * MIN
        }]
    );
}

#[test]
fn us4_cas10_ecriture_en_retard_rien_a_signaler() {
    let lot = [ecr_l(10 * HEURE - 2 * HEURE, 7)];
    assert!(avances(&lot, 10 * HEURE, SEUIL_DEFAUT).is_empty());
}

#[test]
fn avances_maximum_par_appareil_tri_par_appareil() {
    let m = 10 * HEURE;
    let lot = [
        ecr_l(m + 3 * HEURE, 9),
        ecr_l(m + 2 * HEURE, 3),
        ecr_l(m + 5 * HEURE, 9),
    ];
    assert_eq!(
        avances(&lot, m, SEUIL_DEFAUT),
        vec![
            Avance {
                appareil: app(3),
                avance_ms: 2 * HEURE
            },
            Avance {
                appareil: app(9),
                avance_ms: 5 * HEURE
            },
        ]
    );
}

#[test]
fn borne_compteur_sature_et_heure_figee() {
    let mut h = Horloge::nouvelle(app(1));
    h.recevoir(&marque(100, u32::MAX, 2));
    let avant = h.clone();
    assert_eq!(h.emettre(100), Err(HorlogeEpuisee));
    assert_eq!(h, avant);
    assert_eq!(h.emettre(101).unwrap(), marque(101, 0, 1));
}

#[test]
fn borne_l_max_atteinte() {
    let mut h = Horloge::nouvelle(app(1));
    let avant = h.clone();
    assert_eq!(h.emettre(L_MAX), Err(HorlogeEpuisee));
    assert_eq!(h, avant);
}

#[test]
fn borne_l_max_moins_un_emet() {
    let mut h = Horloge::nouvelle(app(1));
    assert_eq!(h.emettre(L_MAX - 1).unwrap(), marque(L_MAX - 1, 0, 1));
}

#[test]
fn id_appareil_conserve_l_ordre_des_entiers() {
    assert!(app(1) < app(2));
    assert!(app(255) < app(256));
    assert!(app(u128::MAX - 1) < app(u128::MAX));
}

#[test]
fn horloge_simulee_fixer_avancer_reculer() {
    let s = HorlogeSimulee::nouvelle(1_000);
    assert_eq!(s.now(), 1_000);
    s.avancer(500);
    assert_eq!(s.now(), 1_500);
    s.reculer(1_200);
    assert_eq!(s.now(), 300);
    s.fixer(42);
    assert_eq!(s.now(), 42);
}

#[test]
fn horloge_simulee_figer() {
    let s = HorlogeSimulee::nouvelle(1_000);
    s.figer();
    s.avancer(500);
    s.reculer(200);
    assert_eq!(s.now(), 1_000);
    s.fixer(2_000);
    s.avancer(1);
    assert_eq!(s.now(), 2_001);
}

fn une_marque() -> impl Strategy<Value = Marque> {
    (0u64..4, 0u32..3, 0u128..3).prop_map(|(l, c, a)| marque(l, c, a))
}

/// Suites d'heures croissantes, figées, décroissantes ou mélangées (INV-ST01-5).
fn des_heures() -> impl Strategy<Value = Vec<u64>> {
    let brut = prop::collection::vec(0u64..10_000, 1..50);
    prop_oneof![
        brut.clone().prop_map(|mut v| {
            v.sort();
            v
        }),
        (0u64..10_000, 1usize..50).prop_map(|(t, n)| vec![t; n]),
        brut.clone().prop_map(|mut v| {
            v.sort_by(|a, b| b.cmp(a));
            v
        }),
        brut,
    ]
}

proptest! {
    #![proptest_config(config())]

    /// US-4 cas 6, INV-ST01-4 : ordre total, lexicographique sur `(l, c, a)`.
    #[test]
    fn inv_st01_4_ordre_total(m1 in une_marque(), m2 in une_marque()) {
        prop_assert_eq!(m1.cmp(&m2), (m1.l, m1.c, m1.a).cmp(&(m2.l, m2.c, m2.a)));
        prop_assert_eq!(m1 == m2, m1.cmp(&m2).is_eq());
        prop_assert_eq!(m1.cmp(&m2), m2.cmp(&m1).reverse());
    }

    /// INV-ST01-5 : émissions strictement croissantes, `l ≥ pt`.
    #[test]
    fn inv_st01_5_emissions_croissantes(heures in des_heures()) {
        let mut h = Horloge::nouvelle(app(1));
        let mut precedente: Option<Marque> = None;
        for pt in heures {
            let m = h.emettre(pt).unwrap();
            prop_assert!(m.l >= pt);
            if let Some(p) = precedente {
                prop_assert!(m > p);
            }
            precedente = Some(m);
        }
    }

    /// INV-ST01-6 : après `recevoir(m)`, toute émission est `> m`.
    #[test]
    fn inv_st01_6_reception(avant in des_heures(), l in 0u64..20_000, c in 0u32..1_000, a in 0u128..5, pt in 0u64..20_000) {
        let mut h = Horloge::nouvelle(app(2));
        for t in avant {
            h.emettre(t).unwrap();
        }
        let m = marque(l, c, a);
        h.recevoir(&m);
        prop_assert!(h.emettre(pt).unwrap() > m);
    }

    /// INV-ST01-8 : seul ce qui dépasse `σ` est signalé, avance maximale par appareil, tri par appareil.
    #[test]
    fn inv_st01_8_avances(ls in prop::collection::vec((0u64..200, 0u128..4), 0..20), maintenant in 0u64..100, s in 1u64..60) {
        let lot: Vec<Ecriture> = ls.iter().map(|&(l, a)| ecr_l(l, a)).collect();
        let rendu = avances(&lot, maintenant, seuil(s));
        prop_assert!(rendu.windows(2).all(|p| p[0].appareil < p[1].appareil));
        for a in 0u128..4 {
            let max = ls.iter().filter(|&&(_, b)| b == a).map(|&(l, _)| l.saturating_sub(maintenant)).max();
            let attendu = max.filter(|&av| av > s).map(|av| Avance { appareil: app(a), avance_ms: av });
            prop_assert_eq!(rendu.iter().find(|x| x.appareil == app(a)).copied(), attendu);
        }
    }
}
