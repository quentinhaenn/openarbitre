//! Support des tests du cœur : configuration proptest, constructeurs, appareils simulés et oracle (T-3, T-7).
#![allow(dead_code)]

use arbitre_core::*;
use proptest::test_runner::{Config as ProptestConfig, RngSeed};

/// Graine fixe des propriétés : la changer passe par une PR (F-3).
pub const GRAINE: u64 = 0x5701_0001;

/// Configuration unique de toutes les propriétés : 1 000 jeux, graine fixe (MANQUE-ST01-7, F-3).
pub fn config() -> ProptestConfig {
    ProptestConfig {
        cases: 1000,
        rng_seed: RngSeed::Fixed(GRAINE),
        ..ProptestConfig::default()
    }
}

pub fn app(n: u128) -> IdAppareil {
    IdAppareil::depuis_u128(n)
}

pub fn marque(l: u64, c: u32, a: u128) -> Marque {
    Marque { l, c, a: app(a) }
}

pub fn donnee(id: &str) -> IdDonnee {
    IdDonnee {
        module: "m".into(),
        genre: "g".into(),
        id: id.into(),
    }
}

pub fn champ(nom: &str, valeur: &str) -> Quoi {
    Quoi::Champ {
        nom: Champ(nom.into()),
        valeur: Valeur(valeur.as_bytes().to_vec()),
    }
}

pub fn visee(nom: &str) -> Visee {
    Visee::Champ(Champ(nom.into()))
}

/// Écriture construite à la main (lot forgé), `vu` vide sauf mention.
pub fn ecriture(donnee: IdDonnee, quoi: Quoi, marque: Marque) -> Ecriture {
    Ecriture {
        donnee,
        quoi,
        marque,
        vu: Vu::new(),
        auteur: Auteur::UtilisateurCli,
    }
}

#[cfg(feature = "sim")]
#[allow(unused_imports)]
pub use sim::*;

/// Appareils simulés et oracle causal (T-7, F-2).
#[cfg(feature = "sim")]
mod sim {
    use std::collections::{BTreeMap, BTreeSet};

    use super::{app, champ, donnee};
    use arbitre_core::*;
    use proptest::prelude::*;

    /// Identité d'une écriture `(cible, marque)`.
    pub type Identite = (Cible, Marque);

    /// Registre de vérité : pour chaque écriture émise, les identités connues de son auteur juste avant `ecrire`.
    pub type Registre = BTreeMap<Identite, BTreeSet<Identite>>;

    /// Un appareil = un journal, une source d'heure et le registre de ses émissions.
    pub struct Appareil {
        pub journal: Journal,
        pub horloge: HorlogeSimulee,
        pub connu: Registre,
    }

    /// Action d'un scénario. Indices réduits modulo le nombre d'appareils.
    #[derive(Debug, Clone)]
    pub enum Action {
        /// `donnee ∈ {0, 1}`, `visee ∈ {0: f1, 1: f2, 2: suppression}`, `valeur ∈ {0: a, 1: b}`, `delta_pt ∈ −2..=2`.
        Ecrire {
            i: usize,
            donnee: u8,
            visee: u8,
            valeur: u8,
            delta_pt: i8,
        },
        /// Livre à `a` le sous-ensemble `selection` du journal de `de`, mélangé selon `ordre`.
        Livrer {
            de: usize,
            a: usize,
            selection: Vec<bool>,
            ordre: u64,
        },
        /// Relivre le dernier lot envoyé de `de` à `a`.
        Relivrer { de: usize, a: usize },
    }

    /// Scénario : 3 à 5 appareils, décalage d'heure initial par appareil, suite d'actions.
    #[derive(Debug, Clone)]
    pub struct Scenario {
        pub decalages: Vec<u64>,
        pub actions: Vec<Action>,
    }

    /// Familles d'E-1.1 à couvrir (R-7).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub enum Famille {
        LotVide,
        Doublon,
        TroisAppareils,
        MemeL,
        MemeLCDeuxAppareils,
        Suppression,
        ConcurrentesMemeValeur,
        EcrasementSequentiel,
        ContreExempleF1,
    }

    /// Heure de départ de chaque source, avant décalage.
    const BASE: u64 = 100;

    /// Générateur de scénarios (E-1.1, R-7) : petits domaines, collisions fréquentes.
    pub fn scenario() -> BoxedStrategy<Scenario> {
        let ecrire = (0usize..5, 0u8..2, 0u8..3, 0u8..2, -2i8..=2).prop_map(
            |(i, donnee, visee, valeur, delta_pt)| Action::Ecrire {
                i,
                donnee,
                visee,
                valeur,
                delta_pt,
            },
        );
        let livrer = (
            0usize..5,
            0usize..5,
            prop::collection::vec(any::<bool>(), 1..8),
            any::<u64>(),
        )
            .prop_map(|(de, a, selection, ordre)| Action::Livrer {
                de,
                a,
                selection,
                ordre,
            });
        let relivrer = (0usize..5, 0usize..5).prop_map(|(de, a)| Action::Relivrer { de, a });
        let action = prop_oneof![6 => ecrire, 3 => livrer, 1 => relivrer];
        (3usize..=5)
            .prop_flat_map(move |n| {
                (
                    prop::collection::vec(0u64..4, n),
                    prop::collection::vec(action.clone(), 4..24),
                )
            })
            .prop_map(|(decalages, actions)| Scenario { decalages, actions })
            .boxed()
    }

    /// Exécution d'un scénario.
    pub struct Simulation {
        pub appareils: Vec<Appareil>,
        derniers: BTreeMap<(usize, usize), Vec<Ecriture>>,
        evenements: BTreeSet<Famille>,
    }

    fn identites(journal: &Journal) -> BTreeSet<Identite> {
        journal
            .ecritures()
            .iter()
            .map(|w| (w.cible(), w.marque))
            .collect()
    }

    /// `y ∈ connu` sur la cible de `x` avec `marque(y) ≥ marque(x)`.
    fn connait(connu: &BTreeSet<Identite>, x: &Ecriture) -> bool {
        let cible = x.cible();
        connu
            .range((cible.clone(), x.marque)..)
            .next()
            .is_some_and(|(c, _)| *c == cible)
    }

    impl Simulation {
        pub fn nouvelle(sc: &Scenario) -> Self {
            let appareils = sc
                .decalages
                .iter()
                .enumerate()
                .map(|(i, d)| Appareil {
                    journal: Journal::nouveau(app(i as u128 + 1)),
                    horloge: HorlogeSimulee::nouvelle(BASE + d),
                    connu: Registre::new(),
                })
                .collect();
            Self {
                appareils,
                derniers: BTreeMap::new(),
                evenements: BTreeSet::new(),
            }
        }

        /// Applique une action ; rend les rapports de fusion produits, par appareil récepteur.
        pub fn appliquer(&mut self, action: &Action) -> Vec<(usize, Rapport)> {
            let n = self.appareils.len();
            match action {
                Action::Ecrire {
                    i,
                    donnee: d,
                    visee,
                    valeur,
                    delta_pt,
                } => {
                    let appareil = &mut self.appareils[i % n];
                    match delta_pt {
                        1.. => appareil.horloge.avancer(*delta_pt as u64),
                        _ => appareil.horloge.reculer(delta_pt.unsigned_abs() as u64),
                    }
                    let quoi = match visee {
                        2 => Quoi::Suppression,
                        v => champ(["f1", "f2"][*v as usize], ["a", "b"][*valeur as usize]),
                    };
                    let connu = identites(&appareil.journal);
                    let pt = appareil.horloge.now();
                    let w = appareil.journal.ecrire(
                        pt,
                        donnee(["d0", "d1"][*d as usize]),
                        quoi,
                        Auteur::UtilisateurCli,
                    );
                    let w = w.expect("écriture du scénario valide");
                    appareil.connu.insert((w.cible(), w.marque), connu);
                    Vec::new()
                }
                Action::Livrer {
                    de,
                    a,
                    selection,
                    ordre,
                } => {
                    let (de, a) = (de % n, (de % n + 1 + a % (n - 1)) % n);
                    let mut lot: Vec<Ecriture> = self.appareils[de]
                        .journal
                        .ecritures()
                        .into_iter()
                        .enumerate()
                        .filter(|(k, _)| selection[k % selection.len()])
                        .map(|(_, w)| w)
                        .collect();
                    let mut r = *ordre;
                    for k in (1..lot.len()).rev() {
                        r = r
                            .wrapping_mul(6364136223846793005)
                            .wrapping_add(1442695040888963407);
                        lot.swap(k, (r >> 33) as usize % (k + 1));
                    }
                    self.derniers.insert((de, a), lot.clone());
                    vec![self.livrer(a, &lot)]
                }
                Action::Relivrer { de, a } => {
                    let (de, a) = (de % n, (de % n + 1 + a % (n - 1)) % n);
                    let lot = self.derniers.get(&(de, a)).cloned().unwrap_or_default();
                    vec![self.livrer(a, &lot)]
                }
            }
        }

        fn livrer(&mut self, a: usize, lot: &[Ecriture]) -> (usize, Rapport) {
            let journal = &mut self.appareils[a].journal;
            let connues = journal.ecritures();
            if lot.is_empty() {
                self.evenements.insert(Famille::LotVide);
            }
            if lot.iter().any(|w| connues.contains(w)) {
                self.evenements.insert(Famille::Doublon);
            }
            (a, journal.fusionner(lot).expect("lot du simulateur valide"))
        }

        /// Livre tout à tous ; rend les rapports produits.
        pub fn tout_livrer(&mut self) -> Vec<(usize, Rapport)> {
            let n = self.appareils.len();
            let mut rapports = Vec::new();
            for de in 0..n {
                for a in (0..n).filter(|a| *a != de) {
                    let lot = self.appareils[de].journal.ecritures();
                    rapports.push(self.livrer(a, &lot));
                }
            }
            rapports
        }

        /// Union des registres de tous les appareils.
        pub fn registre(&self) -> Registre {
            self.appareils
                .iter()
                .flat_map(|a| a.connu.clone())
                .collect()
        }

        /// Familles d'E-1.1 rencontrées depuis le début de la simulation.
        pub fn familles(&self) -> BTreeSet<Famille> {
            let mut familles = self.evenements.clone();
            let registre = self.registre();
            let s: BTreeSet<Ecriture> = self
                .appareils
                .iter()
                .flat_map(|a| a.journal.ecritures())
                .collect();
            let s: Vec<Ecriture> = s.into_iter().collect();
            if self.appareils.len() >= 3 {
                familles.insert(Famille::TroisAppareils);
            }
            if s.iter().any(|w| matches!(w.quoi, Quoi::Suppression)) {
                familles.insert(Famille::Suppression);
            }
            for (k, w) in s.iter().enumerate() {
                for x in &s[k + 1..] {
                    if w.marque == x.marque {
                        continue;
                    }
                    if w.marque.l == x.marque.l {
                        familles.insert(Famille::MemeL);
                        if w.marque.c == x.marque.c {
                            familles.insert(Famille::MemeLCDeuxAppareils);
                        }
                    }
                    if !exclusives(w, x) {
                        continue;
                    }
                    let (wx, xw) = (
                        connait(&registre[&(w.cible(), w.marque)], x),
                        connait(&registre[&(x.cible(), x.marque)], w),
                    );
                    if wx != xw && w.quoi != x.quoi {
                        familles.insert(Famille::EcrasementSequentiel);
                    }
                    if !wx && !xw {
                        if w.quoi == x.quoi {
                            familles.insert(Famille::ConcurrentesMemeValeur);
                        }
                        let (champ, suppression) = if matches!(w.quoi, Quoi::Suppression) {
                            (x, w)
                        } else {
                            (w, x)
                        };
                        let autre_champ = registre[&(suppression.cible(), suppression.marque)]
                            .iter()
                            .any(|(c, m)| {
                                c.donnee == champ.donnee
                                    && c.visee != champ.cible().visee
                                    && *c != suppression.cible()
                                    && *m > champ.marque
                            });
                        if matches!(suppression.quoi, Quoi::Suppression) && autre_champ {
                            familles.insert(Famille::ContreExempleF1);
                        }
                    }
                }
            }
            familles
        }
    }

    /// Même donnée, et même champ ou suppression contre champ.
    fn exclusives(w: &Ecriture, x: &Ecriture) -> bool {
        w.donnee == x.donnee
            && match (&w.quoi, &x.quoi) {
                (Quoi::Champ { nom: a, .. }, Quoi::Champ { nom: b, .. }) => a == b,
                (Quoi::Suppression, Quoi::Suppression) => false,
                _ => true,
            }
    }

    fn rang(w: &Ecriture) -> (Marque, Cible, Option<Valeur>, Vu, Auteur) {
        let valeur = match &w.quoi {
            Quoi::Champ { valeur, .. } => Some(valeur.clone()),
            Quoi::Suppression => None,
        };
        (w.marque, w.cible(), valeur, w.vu.clone(), w.auteur)
    }

    /// Oracle (F-2) : conflits de `s` par force brute sur toutes les paires, d'après le seul registre.
    /// `a_vu*(w, x) ⇔ ∃ y ∈ connu(w), cible(y) = cible(x) ∧ marque(y) ≥ marque(x)` ; normalisés et triés (A-4).
    pub fn attendus(s: &[Ecriture], registre: &Registre) -> Vec<Conflit> {
        let mut conflits = Vec::new();
        for (k, w) in s.iter().enumerate() {
            for x in &s[k + 1..] {
                if !exclusives(w, x) || w.quoi == x.quoi {
                    continue;
                }
                let a_vu =
                    |a: &Ecriture, b: &Ecriture| connait(&registre[&(a.cible(), a.marque)], b);
                if a_vu(w, x) || a_vu(x, w) {
                    continue;
                }
                let (retenue, ecartee) = if rang(w) > rang(x) { (w, x) } else { (x, w) };
                let ((Quoi::Champ { nom, .. }, _) | (_, Quoi::Champ { nom, .. })) =
                    (&w.quoi, &x.quoi)
                else {
                    continue;
                };
                conflits.push(Conflit {
                    donnee: w.donnee.clone(),
                    champ: nom.clone(),
                    retenue: retenue.clone(),
                    ecartee: ecartee.clone(),
                });
            }
        }
        let cle = |k: &Conflit| {
            (
                k.donnee.clone(),
                k.champ.clone(),
                k.retenue.marque,
                k.ecartee.marque,
                rang(&k.retenue),
                rang(&k.ecartee),
            )
        };
        conflits.sort_by_key(cle);
        conflits
    }
}
