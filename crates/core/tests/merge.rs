//! Journal, fusion, projection, écartées, anomalies (T-5) ; conflits et tris (T-6).
//! Matrices US-1, US-2, US-3 ; A-1 à A-4, A-10.

mod support;

use std::collections::BTreeSet;
use std::time::Instant;

use arbitre_core::*;
use proptest::prelude::*;
use support::*;

const A: u128 = 1;
const B: u128 = 2;
const C: u128 = 3;
const X: u128 = 9;

fn j(n: u128) -> Journal {
    Journal::nouveau(app(n))
}

fn ecr(journal: &mut Journal, pt: u64, d: &str, quoi: Quoi) -> Ecriture {
    journal
        .ecrire(pt, donnee(d), quoi, Auteur::UtilisateurCli)
        .unwrap()
}

fn recu(lot: &[Ecriture]) -> Journal {
    let mut x = j(X);
    x.fusionner(lot).unwrap();
    x
}

fn valeur_de(x: &Journal, d: &str, f: &str) -> Quoi {
    x.projection()[&donnee(d)].champs[&Champ(f.into())]
        .quoi
        .clone()
}

fn statut(x: &Journal, d: &str) -> Statut {
    x.projection()[&donnee(d)].statut.clone()
}

fn conflit(retenue: &Ecriture, ecartee: &Ecriture, f: &str) -> Conflit {
    Conflit {
        donnee: retenue.donnee.clone(),
        champ: Champ(f.into()),
        retenue: retenue.clone(),
        ecartee: ecartee.clone(),
    }
}

fn r_de(x: &Journal) -> (Projection, Vec<Ecriture>, Vec<Conflit>) {
    (x.projection(), x.ecartees(), x.conflits())
}

// ---- T-5 : matrice US-1 ----

#[test]
fn us1_cas1_champs_differents_deux_appareils() {
    let wa = ecr(&mut j(A), 10, "d", champ("titre", "v1"));
    let wb = ecr(&mut j(B), 12, "d", champ("echeance", "v2"));
    let x = recu(&[wa, wb]);
    assert_eq!(valeur_de(&x, "d", "titre"), champ("titre", "v1"));
    assert_eq!(valeur_de(&x, "d", "echeance"), champ("echeance", "v2"));
    assert_eq!(statut(&x, "d"), Statut::Presente);
    assert!(x.ecartees().is_empty());
}

#[test]
fn us1_cas2_meme_champ_marques_differentes() {
    let wa = ecr(&mut j(A), 10, "d", champ("f", "v1"));
    let wb = ecr(&mut j(B), 12, "d", champ("f", "v2"));
    let x = recu(&[wa.clone(), wb]);
    assert_eq!(valeur_de(&x, "d", "f"), champ("f", "v2"));
    assert_eq!(x.ecartees(), vec![wa]);
}

#[test]
fn us1_cas3_meme_l_c_departage_par_appareil() {
    let wa = ecr(&mut j(A), 10, "d", champ("f", "v1"));
    let wb = ecr(&mut j(B), 10, "d", champ("f", "v2"));
    assert_eq!((wa.marque.l, wa.marque.c), (wb.marque.l, wb.marque.c));
    for lot in [[wa.clone(), wb.clone()], [wb.clone(), wa.clone()]] {
        let x = recu(&lot);
        assert_eq!(valeur_de(&x, "d", "f"), champ("f", "v2"));
        assert_eq!(x.ecartees(), vec![wa.clone()]);
    }
}

#[test]
fn us1_cas7_lot_vide() {
    let mut x = recu(&[ecr(&mut j(A), 10, "d", champ("f", "v1"))]);
    let avant = (r_de(&x), x.anomalies());
    assert_eq!(x.fusionner(&[]).unwrap(), Rapport::default());
    assert_eq!((r_de(&x), x.anomalies()), avant);
}

#[test]
fn us1_cas8_dix_mille_ecritures_sequentielles() {
    let mut a = j(A);
    for k in 0..10_000u64 {
        ecr(&mut a, k / 3, "d", champ("f", &k.to_string()));
    }
    assert_eq!(valeur_de(&a, "d", "f"), champ("f", "9999"));
    assert_eq!(a.ecartees().len(), 9_999);
}

#[test]
fn us1_cas10_collision_d_identite() {
    let w1 = ecriture(donnee("d"), champ("f", "v1"), marque(10, 0, A));
    let w2 = ecriture(donnee("d"), champ("f", "v2"), marque(10, 0, A));
    let mut x = recu(std::slice::from_ref(&w1));
    let rapport = x.fusionner(std::slice::from_ref(&w2)).unwrap();
    assert_eq!(x.ecritures(), vec![w1.clone(), w2.clone()]);
    assert_eq!(valeur_de(&x, "d", "f"), champ("f", "v2"));
    assert_eq!(x.ecartees(), vec![w1.clone()]);
    assert_eq!(rapport.nouvelles_anomalies.len(), 1);
    assert_eq!(x.anomalies().len(), 1);
    let avant = r_de(&x);
    let seconde = x.fusionner(&[w1]).unwrap();
    assert!(seconde.nouvelles_anomalies.is_empty());
    assert_eq!(r_de(&x), avant);
}

#[test]
fn us1_cas11_lot_mal_forme_refuse_en_entier() {
    let mut x = recu(&[ecr(&mut j(A), 10, "d", champ("f", "v1"))]);
    let avant = x.clone();
    let bonne = ecr(&mut j(B), 20, "d", champ("f", "v2"));
    let mauvaise = ecriture(donnee("d"), champ("", "v3"), marque(21, 0, B));
    assert_eq!(
        x.fusionner(&[bonne, mauvaise]),
        Err(ErreurLot {
            index: 1,
            raison: Raison::IdentifiantVide
        })
    );
    assert_eq!(x, avant);
}

// ---- T-5 : matrice US-3 (état et écartées) ----

#[test]
fn us3_cas1_suppression_puis_modification_concurrente() {
    let s = ecr(&mut j(A), 10, "d", Quoi::Suppression);
    let w = ecr(&mut j(B), 12, "d", champ("f", "v"));
    let x = recu(&[s.clone(), w]);
    assert_eq!(statut(&x, "d"), Statut::Presente);
    assert_eq!(valeur_de(&x, "d", "f"), champ("f", "v"));
    assert_eq!(x.ecartees(), vec![s]);
}

#[test]
fn us3_cas2_modification_puis_suppression_concurrente() {
    let w = ecr(&mut j(A), 10, "d", champ("f", "v"));
    let s = ecr(&mut j(B), 12, "d", Quoi::Suppression);
    let x = recu(&[w, s.clone()]);
    assert_eq!(statut(&x, "d"), Statut::Supprimee(s));
    assert_eq!(valeur_de(&x, "d", "f"), champ("f", "v"));
    assert!(x.ecartees().is_empty());
}

#[test]
fn us3_cas3_suppression_sequentielle() {
    let w = ecr(&mut j(A), 10, "d", champ("f", "v"));
    let mut b = recu(&[w]);
    let s = ecr(&mut b, 11, "d", Quoi::Suppression);
    assert_eq!(statut(&b, "d"), Statut::Supprimee(s));
}

#[test]
fn us3_cas4_resurrection_sequentielle() {
    let s = ecr(&mut j(A), 10, "d", Quoi::Suppression);
    let mut b = recu(std::slice::from_ref(&s));
    ecr(&mut b, 11, "d", champ("f", "v"));
    assert_eq!(statut(&b, "d"), Statut::Presente);
    assert_eq!(b.ecartees(), vec![s]);
}

#[test]
fn us3_cas5_meme_l_c_suppression_contre_champ() {
    let s = ecr(&mut j(A), 10, "d", Quoi::Suppression);
    let w = ecr(&mut j(B), 10, "d", champ("f", "v"));
    for lot in [[s.clone(), w.clone()], [w.clone(), s.clone()]] {
        assert_eq!(statut(&recu(&lot), "d"), Statut::Presente);
    }
}

#[test]
fn us3_cas6_double_suppression_concurrente() {
    let sa = ecr(&mut j(A), 10, "d", Quoi::Suppression);
    let sb = ecr(&mut j(B), 12, "d", Quoi::Suppression);
    let x = recu(&[sa.clone(), sb.clone()]);
    assert_eq!(statut(&x, "d"), Statut::Supprimee(sb));
    assert_eq!(x.ecartees(), vec![sa]);
}

#[test]
fn us3_cas7_resurrection_tous_champs() {
    let mut a = j(A);
    let f1a = ecr(&mut a, 5, "d", champ("f1", "a"));
    ecr(&mut a, 6, "d", champ("f2", "b"));
    let s = ecr(&mut a, 8, "d", Quoi::Suppression);
    ecr(&mut a, 9, "d", champ("f1", "c"));
    assert_eq!(statut(&a, "d"), Statut::Presente);
    assert_eq!(valeur_de(&a, "d", "f1"), champ("f1", "c"));
    assert_eq!(valeur_de(&a, "d", "f2"), champ("f2", "b"));
    assert_eq!(a.ecartees(), vec![s, f1a]);
}

#[test]
fn us3_cas9_suppression_seule() {
    let s = ecr(&mut j(A), 10, "d", Quoi::Suppression);
    let x = recu(std::slice::from_ref(&s));
    assert_eq!(statut(&x, "d"), Statut::Supprimee(s));
    assert!(x.projection()[&donnee("d")].champs.is_empty());
}

#[test]
fn us2_cas9_ecartee_complete() {
    let wa = Ecriture {
        auteur: Auteur::Assistant,
        ..ecr(&mut j(A), 10, "d", champ("f", "v1"))
    };
    let wb = ecr(&mut j(B), 12, "d", champ("f", "v2"));
    let ecartee = &recu(&[wa.clone(), wb]).ecartees()[0];
    assert_eq!(ecartee, &wa);
    assert_eq!(ecartee.appareil(), app(A));
}

// ---- T-5 : écriture locale et horloge du journal (A-1, A-2) ----

#[test]
fn ecrire_calcule_vu_par_visee() {
    let mut a = j(A);
    let w1 = ecr(&mut a, 1, "d", champ("f1", "a"));
    let w2 = ecr(&mut a, 2, "d", champ("f2", "b"));
    let s = ecr(&mut a, 3, "d", Quoi::Suppression);
    let w4 = ecr(&mut a, 4, "d", champ("f1", "c"));
    assert!(w1.vu.is_empty());
    assert!(w2.vu.is_empty());
    assert_eq!(
        s.vu,
        Vu::from([(visee("f1"), w1.marque), (visee("f2"), w2.marque)])
    );
    assert_eq!(
        w4.vu,
        Vu::from([(Visee::Suppression, s.marque), (visee("f1"), w1.marque)])
    );
}

#[test]
fn ecrire_ignore_les_autres_donnees() {
    let mut a = j(A);
    ecr(&mut a, 1, "autre", champ("f", "a"));
    ecr(&mut a, 2, "autre", Quoi::Suppression);
    assert!(ecr(&mut a, 3, "d", champ("f", "b")).vu.is_empty());
}

#[test]
fn ecrire_identifiant_vide_ne_consomme_pas_de_marque() {
    let mut a = j(A);
    assert_eq!(
        ecr(&mut a, 10, "d", champ("f", "a")).marque,
        marque(10, 0, A)
    );
    let avant = a.clone();
    assert_eq!(
        a.ecrire(10, donnee(""), champ("f", "b"), Auteur::Systeme),
        Err(ErreurEcriture::Invalide(Raison::IdentifiantVide))
    );
    assert_eq!(a, avant);
    assert_eq!(
        ecr(&mut a, 10, "d", champ("f", "c")).marque,
        marque(10, 1, A)
    );
}

#[test]
fn ecrire_horloge_epuisee() {
    let mut a = j(A);
    a.fusionner(&[ecriture(
        donnee("d"),
        champ("f", "v"),
        marque(10, u32::MAX, B),
    )])
    .unwrap();
    assert_eq!(
        a.ecrire(10, donnee("d"), champ("f", "w"), Auteur::Systeme),
        Err(ErreurEcriture::HorlogeEpuisee)
    );
}

#[test]
fn fusionner_lot_invalide_journal_et_horloge_inchanges() {
    let mut a = j(A);
    ecr(&mut a, 10, "d", champ("f", "a"));
    let avant = a.clone();
    let lot = [
        ecriture(donnee("d"), champ("f", "b"), marque(1_000, 0, B)),
        ecriture(donnee("d"), champ("f", "c"), marque(L_MAX, 0, B)),
    ];
    assert_eq!(
        a.fusionner(&lot),
        Err(ErreurLot {
            index: 1,
            raison: Raison::MarqueHorsBornes
        })
    );
    assert_eq!(a, avant);
    assert_eq!(
        ecr(&mut a, 10, "d", champ("f", "d")).marque,
        marque(10, 1, A)
    );
}

#[test]
fn fusionner_lot_valide_l_horloge_adopte_le_maximum() {
    let mut a = j(A);
    let lot = [
        ecriture(donnee("d"), champ("f", "b"), marque(500, 4, B)),
        ecriture(donnee("e"), champ("f", "c"), marque(900, 2, C)),
    ];
    a.fusionner(&lot).unwrap();
    assert!(ecr(&mut a, 10, "d", champ("f", "x")).marque > marque(900, 2, C));
}

// ---- T-6 : conflits (matrices US-2, US-3) ----

#[test]
fn us2_cas1_meme_champ_hors_ligne_des_deux_cotes() {
    let wa = ecr(&mut j(A), 10, "d", champ("f", "v1"));
    let wb = ecr(&mut j(B), 12, "d", champ("f", "v2"));
    let mut x = j(X);
    let rapport = x.fusionner(&[wa.clone(), wb.clone()]).unwrap();
    assert_eq!(x.conflits(), vec![conflit(&wb, &wa, "f")]);
    assert_eq!(rapport.nouveaux_conflits, x.conflits());
    assert_eq!(rapport.nouvelles_ecartees, vec![wa]);
}

#[test]
fn us2_cas2_ecrasement_sequentiel() {
    let wa = ecr(&mut j(A), 10, "d", champ("f", "v1"));
    let mut b = recu(std::slice::from_ref(&wa));
    let wb = ecr(&mut b, 11, "d", champ("f", "v2"));
    assert!(b.conflits().is_empty());
    assert!(recu(&[wb, wa]).conflits().is_empty());
}

#[test]
fn us2_cas3_ecrasement_sequentiel_recepteur_en_retard() {
    let mut a = j(A);
    let mut b = j(B);
    let w0 = ecr(&mut b, 5, "d", champ("f", "v0"));
    a.fusionner(&[w0]).unwrap();
    let wa = ecr(&mut a, 10, "d", champ("f", "v1"));
    b.fusionner(&[wa]).unwrap();
    let wb = ecr(&mut b, 11, "d", champ("f", "v2"));
    let rapport = a.fusionner(&[wb]).unwrap();
    assert!(rapport.nouveaux_conflits.is_empty());
    assert!(a.conflits().is_empty());
}

#[test]
fn us2_cas4_concurrentes_de_meme_valeur() {
    let wa = ecr(&mut j(A), 10, "d", champ("f", "v"));
    let wb = ecr(&mut j(B), 12, "d", champ("f", "v"));
    let x = recu(&[wa.clone(), wb.clone()]);
    assert_eq!(x.projection()[&donnee("d")].champs[&Champ("f".into())], wb);
    assert_eq!(x.ecartees(), vec![wa]);
    assert!(x.conflits().is_empty());
}

#[test]
fn us2_cas6_trois_concurrentes_un_conflit_par_paire() {
    let wa = ecr(&mut j(A), 10, "d", champ("f", "v1"));
    let wb = ecr(&mut j(B), 11, "d", champ("f", "v2"));
    let wc = ecr(&mut j(C), 12, "d", champ("f", "v3"));
    let x = recu(&[wc.clone(), wa.clone(), wb.clone()]);
    assert_eq!(valeur_de(&x, "d", "f"), champ("f", "v3"));
    assert_eq!(x.ecartees(), vec![wa.clone(), wb.clone()]);
    assert_eq!(
        x.conflits(),
        vec![
            conflit(&wb, &wa, "f"),
            conflit(&wc, &wa, "f"),
            conflit(&wc, &wb, "f")
        ]
    );
}

#[test]
fn us2_cas7_pair_jamais_synchronise() {
    let wp = ecr(&mut j(B), 10, "d", champ("f", "v1"));
    let mut local = j(A);
    let wl = ecr(&mut local, 20, "d", champ("f", "v2"));
    let rapport = local.fusionner(std::slice::from_ref(&wp)).unwrap();
    assert_eq!(rapport.nouveaux_conflits, vec![conflit(&wl, &wp, "f")]);
}

#[test]
fn us2_cas8_rejeu_memes_listes_meme_ordre() {
    let lot = [
        ecr(&mut j(A), 10, "d", champ("f", "v1")),
        ecr(&mut j(B), 12, "d", champ("f", "v2")),
        ecr(&mut j(C), 11, "e", Quoi::Suppression),
        ecr(&mut j(A), 9, "e", champ("g", "x")),
    ];
    let (mut x, mut y) = (j(X), j(X));
    assert_eq!(x.fusionner(&lot).unwrap(), y.fusionner(&lot).unwrap());
    assert_eq!(r_de(&x), r_de(&y));
}

#[test]
fn us2_cas10_ecriture_sequentielle_apres_conflit() {
    let wa = ecr(&mut j(A), 10, "d", champ("f", "v1"));
    let wb = ecr(&mut j(B), 12, "d", champ("f", "v2"));
    let mut a = j(A);
    a.fusionner(&[wa.clone(), wb.clone()]).unwrap();
    let w3 = ecr(&mut a, 13, "d", champ("f", "v3"));
    let mut x = recu(&[wa.clone(), wb.clone()]);
    let rapport = x.fusionner(&[w3]).unwrap();
    assert_eq!(valeur_de(&x, "d", "f"), champ("f", "v3"));
    assert_eq!(rapport.nouvelles_ecartees, vec![wb.clone()]);
    assert!(rapport.nouveaux_conflits.is_empty());
    assert_eq!(x.conflits(), vec![conflit(&wb, &wa, "f")]);
}

#[test]
fn us3_cas1_conflit_retenue_champ() {
    let s = ecr(&mut j(A), 10, "d", Quoi::Suppression);
    let w = ecr(&mut j(B), 12, "d", champ("f", "v"));
    assert_eq!(
        recu(&[s.clone(), w.clone()]).conflits(),
        vec![conflit(&w, &s, "f")]
    );
}

#[test]
fn us3_cas2_conflit_retenue_suppression() {
    let w = ecr(&mut j(A), 10, "d", champ("f", "v"));
    let s = ecr(&mut j(B), 12, "d", Quoi::Suppression);
    assert_eq!(
        recu(&[w.clone(), s.clone()]).conflits(),
        vec![conflit(&s, &w, "f")]
    );
}

#[test]
fn us3_cas3_et_4_sequentiels_sans_conflit() {
    let w = ecr(&mut j(A), 10, "d", champ("f", "v"));
    let mut b = recu(&[w]);
    ecr(&mut b, 11, "d", Quoi::Suppression);
    assert!(b.conflits().is_empty());
    let s = ecr(&mut j(A), 10, "e", Quoi::Suppression);
    let mut c = recu(&[s]);
    ecr(&mut c, 11, "e", champ("f", "v"));
    assert!(c.conflits().is_empty());
}

#[test]
fn us3_cas5_meme_l_c_suppression_contre_champ_en_conflit() {
    let s = ecr(&mut j(A), 10, "d", Quoi::Suppression);
    let w = ecr(&mut j(B), 10, "d", champ("f", "v"));
    assert_eq!(
        recu(&[s.clone(), w.clone()]).conflits(),
        vec![conflit(&w, &s, "f")]
    );
}

#[test]
fn us3_cas6_double_suppression_sans_conflit() {
    let sa = ecr(&mut j(A), 10, "d", Quoi::Suppression);
    let sb = ecr(&mut j(B), 12, "d", Quoi::Suppression);
    assert!(recu(&[sa, sb]).conflits().is_empty());
}

#[test]
fn contre_exemple_f1() {
    let titre = ecr(&mut j(B), 10, "d", champ("titre", "t"));
    let mut a = j(A);
    let echeance = ecr(&mut a, 11, "d", champ("echeance", "e"));
    let s = ecr(&mut a, 12, "d", Quoi::Suppression);
    assert_eq!(
        recu(&[titre.clone(), echeance, s.clone()]).conflits(),
        vec![conflit(&s, &titre, "titre")]
    );
}

#[test]
fn cas_f12_meme_marque_suppression_et_champ() {
    let m = marque(10, 0, A);
    let s = ecriture(donnee("d"), Quoi::Suppression, m);
    let w = ecriture(donnee("d"), champ("f", "v"), m);
    let x = recu(&[s.clone(), w.clone()]);
    assert_eq!(statut(&x, "d"), Statut::Presente);
    assert_eq!(x.conflits(), vec![conflit(&w, &s, "f")]);
}

#[test]
fn collision_de_valeurs_differentes_est_un_conflit() {
    let w1 = ecriture(donnee("d"), champ("f", "v1"), marque(10, 0, A));
    let w2 = ecriture(donnee("d"), champ("f", "v2"), marque(10, 0, A));
    assert_eq!(
        recu(&[w1.clone(), w2.clone()]).conflits(),
        vec![conflit(&w2, &w1, "f")]
    );
}

#[test]
fn seconde_reception_trois_listes_vides() {
    let lot = [
        ecr(&mut j(A), 10, "d", champ("f", "v1")),
        ecr(&mut j(B), 12, "d", champ("f", "v2")),
        ecriture(donnee("d"), champ("g", "x"), marque(5, 0, C)),
        ecriture(donnee("d"), champ("g", "y"), marque(5, 0, C)),
    ];
    let mut x = recu(&lot);
    assert_eq!(x.fusionner(&lot).unwrap(), Rapport::default());
}

#[test]
fn us1_cas8_dix_mille_ecritures_en_un_lot_recu() {
    let mut a = j(A);
    let lot: Vec<Ecriture> = (0..10_000u64)
        .map(|k| ecr(&mut a, k / 3, "d", champ("f", &k.to_string())))
        .collect();
    let mut x = j(X);
    let debut = Instant::now();
    let rapport = x.fusionner(&lot).unwrap();
    assert!(debut.elapsed().as_secs_f64() < 2.0);
    assert_eq!(rapport.nouvelles_ecartees.len(), 9_999);
    assert!(rapport.nouveaux_conflits.is_empty());
}

// ---- Propriétés (T-5 partition et monotonie, T-6 tris) ----

/// Écritures forgées sur un petit domaine, `vu` vide : concurrentes, collisions fréquentes (R-7).
fn une_ecriture() -> impl Strategy<Value = Ecriture> {
    (0u8..2, 0u8..3, 0u8..2, 0u64..4, 0u32..2, 1u128..4).prop_map(|(d, v, val, l, c, a)| {
        let d = ["d0", "d1"][d as usize];
        let quoi = match v {
            2 => Quoi::Suppression,
            _ => champ(["f1", "f2"][v as usize], ["a", "b"][val as usize]),
        };
        ecriture(donnee(d), quoi, marque(l, c, a))
    })
}

fn un_lot() -> impl Strategy<Value = Vec<Ecriture>> {
    prop::collection::vec(une_ecriture(), 0..12)
}

fn retenues(x: &Journal) -> BTreeSet<Ecriture> {
    let mut r = BTreeSet::new();
    for etat in x.projection().into_values() {
        r.extend(etat.champs.into_values());
        if let Statut::Supprimee(s) = etat.statut {
            r.insert(s);
        }
    }
    r
}

/// Rang d'une écriture `(marque, cible, contenu)` (A-3).
fn rang(w: &Ecriture) -> impl Ord {
    let valeur = match &w.quoi {
        Quoi::Champ { valeur, .. } => Some(valeur.clone()),
        Quoi::Suppression => None,
    };
    (w.marque, w.cible(), valeur, w.vu.clone(), w.auteur)
}

fn cle_conflit(k: &Conflit) -> impl Ord {
    (
        k.donnee.clone(),
        k.champ.clone(),
        k.retenue.marque,
        k.ecartee.marque,
        rang(&k.retenue),
        rang(&k.ecartee),
    )
}

fn cle_ecartee(w: &Ecriture) -> impl Ord {
    (w.cible(), rang(w))
}

fn strict<T: Ord>(v: Vec<T>) -> bool {
    v.windows(2).all(|p| p[0] < p[1])
}

proptest! {
    #![proptest_config(config())]

    /// INV-7 : `retenues ⊎ écartées = S`.
    #[test]
    fn inv_7_partition(lot in un_lot()) {
        let x = recu(&lot);
        let s: BTreeSet<Ecriture> = x.ecritures().into_iter().collect();
        let e: BTreeSet<Ecriture> = x.ecartees().into_iter().collect();
        let r = retenues(&x);
        prop_assert!(r.is_disjoint(&e));
        prop_assert_eq!(r.union(&e).cloned().collect::<BTreeSet<_>>(), s);
    }

    /// INV-7 : `S ⊆ S' ⇒ écartées(S) ⊆ écartées(S')`.
    #[test]
    fn inv_7_monotonie(a in un_lot(), b in un_lot()) {
        let mut x = recu(&a);
        let avant: BTreeSet<Ecriture> = x.ecartees().into_iter().collect();
        x.fusionner(&b).unwrap();
        let apres: BTreeSet<Ecriture> = x.ecartees().into_iter().collect();
        prop_assert!(avant.is_subset(&apres));
    }

    /// INV-ST01-7 : un lot qui contient une écriture invalide laisse le journal inchangé.
    #[test]
    fn inv_st01_7_lot_invalide(a in un_lot(), b in un_lot(), position in any::<prop::sample::Index>()) {
        let mut x = recu(&a);
        let avant = x.clone();
        let mut lot = b;
        let i = position.index(lot.len() + 1);
        lot.insert(i, ecriture(donnee(""), Quoi::Suppression, marque(1, 0, 1)));
        prop_assert!(x.fusionner(&lot).is_err());
        prop_assert_eq!(x, avant);
    }

    /// INV-ST01-3 : rapports et `conflits()` triés selon A-4, sans doublon.
    #[test]
    fn inv_st01_3_tris(a in un_lot(), b in un_lot()) {
        let mut x = recu(&a);
        let rapport = x.fusionner(&b).unwrap();
        prop_assert!(strict(rapport.nouvelles_ecartees.iter().map(cle_ecartee).collect()));
        prop_assert!(strict(rapport.nouveaux_conflits.iter().map(cle_conflit).collect()));
        prop_assert!(strict(rapport.nouvelles_anomalies.iter().map(|k| (k.cible.clone(), k.marque)).collect()));
        prop_assert!(strict(x.ecartees().iter().map(cle_ecartee).collect()));
        prop_assert!(strict(x.conflits().iter().map(cle_conflit).collect()));
    }
}
