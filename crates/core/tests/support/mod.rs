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
    IdDonnee { module: "m".into(), genre: "g".into(), id: id.into() }
}

pub fn champ(nom: &str, valeur: &str) -> Quoi {
    Quoi::Champ { nom: Champ(nom.into()), valeur: Valeur(valeur.as_bytes().to_vec()) }
}

pub fn visee(nom: &str) -> Visee {
    Visee::Champ(Champ(nom.into()))
}

/// Écriture construite à la main (lot forgé), `vu` vide sauf mention.
pub fn ecriture(donnee: IdDonnee, quoi: Quoi, marque: Marque) -> Ecriture {
    Ecriture { donnee, quoi, marque, vu: Vu::new(), auteur: Auteur::UtilisateurCli }
}

#[cfg(feature = "sim")]
#[allow(unused_imports)]
pub use sim::*;

/// Appareils simulés et oracle causal (T-7, F-2).
#[cfg(feature = "sim")]
mod sim {
    use std::collections::{BTreeMap, BTreeSet};

    use arbitre_core::*;
    use proptest::strategy::BoxedStrategy;

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
        Ecrire { i: usize, donnee: u8, visee: u8, valeur: u8, delta_pt: i8 },
        /// Livre à `a` le sous-ensemble `selection` du journal de `de`, mélangé selon `ordre`.
        Livrer { de: usize, a: usize, selection: Vec<bool>, ordre: u64 },
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

    /// Générateur de scénarios (E-1.1, R-7).
    pub fn scenario() -> BoxedStrategy<Scenario> {
        todo!("T-7: générateur de scénarios — E-1.1")
    }

    /// Exécution d'un scénario.
    pub struct Simulation {
        pub appareils: Vec<Appareil>,
    }

    impl Simulation {
        pub fn nouvelle(sc: &Scenario) -> Self {
            let _ = sc;
            todo!("T-7: Simulation::nouvelle")
        }

        /// Applique une action ; rend les rapports de fusion produits, par appareil récepteur.
        pub fn appliquer(&mut self, action: &Action) -> Vec<(usize, Rapport)> {
            let _ = action;
            todo!("T-7: Simulation::appliquer")
        }

        /// Livre tout à tous ; rend les rapports produits.
        pub fn tout_livrer(&mut self) -> Vec<(usize, Rapport)> {
            todo!("T-7: Simulation::tout_livrer")
        }

        /// Union des registres de tous les appareils.
        pub fn registre(&self) -> Registre {
            todo!("T-7: Simulation::registre")
        }

        /// Familles d'E-1.1 rencontrées depuis le début de la simulation.
        pub fn familles(&self) -> BTreeSet<Famille> {
            todo!("T-7: Simulation::familles — R-7")
        }
    }

    /// Oracle (F-2) : conflits de `s` par force brute sur toutes les paires, d'après le seul registre.
    /// `a_vu*(w, x) ⇔ ∃ y ∈ connu(w), cible(y) = cible(x) ∧ marque(y) ≥ marque(x)` ; normalisés et triés (A-4).
    pub fn attendus(s: &[Ecriture], registre: &Registre) -> Vec<Conflit> {
        let _ = (s, registre);
        todo!("T-7: oracle causal — F-2")
    }
}
