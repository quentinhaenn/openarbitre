//! Écritures et validation de lot (archi ST-01 § Modèle de données, A-1, A-11 ; T-4).
//!
//! Auteur hors liste et appareil ≠ `marque.a` ne sont pas représentables (écart E-1 de l'archi).

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use crate::hlc::{IdAppareil, L_MAX, Marque};

/// Identifiant d'une donnée. Ordre contractuel `(module, genre, id)`, octets UTF-8 (A-4).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct IdDonnee {
    pub module: String,
    pub genre: String,
    pub id: String,
}

/// Nom d'un champ.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Champ(pub String);

/// Valeur sérialisée par le module ; le cœur n'en compare que les octets.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Valeur(pub Vec<u8>);

/// Liste fermée des auteurs (RG-ST01-4.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Auteur {
    UtilisateurCli,
    UtilisateurInterface,
    Assistant,
    Systeme,
}

/// Ce que fait une écriture : affecter un champ ou supprimer la donnée.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Quoi {
    Champ { nom: Champ, valeur: Valeur },
    Suppression,
}

/// Visée d'une écriture dans sa donnée. Ordre contractuel : la suppression avant les champs (A-1, D-4).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Visee {
    Suppression,
    Champ(Champ),
}

/// Cible `(donnee, visee)`. Ordre contractuel (A-4).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cible {
    pub donnee: IdDonnee,
    pub visee: Visee,
}

/// Plus grande marque connue de l'auteur, par visée de la même donnée (A-1). Entrée absente = `⊥`.
pub type Vu = BTreeMap<Visee, Marque>;

/// Écriture `(donnee, quoi, marque, vu, auteur)`. Son `Ord` dérivé n'est pas contractuel (A-4).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ecriture {
    pub donnee: IdDonnee,
    pub quoi: Quoi,
    pub marque: Marque,
    pub vu: Vu,
    pub auteur: Auteur,
}

impl Ecriture {
    /// `(donnee, Champ(nom))` ou `(donnee, Suppression)`.
    pub fn cible(&self) -> Cible {
        let visee = match &self.quoi {
            Quoi::Champ { nom, .. } => Visee::Champ(nom.clone()),
            Quoi::Suppression => Visee::Suppression,
        };
        Cible {
            donnee: self.donnee.clone(),
            visee,
        }
    }

    /// Appareil auteur, `= marque.a` par construction (RG-ST01-4.5).
    pub fn appareil(&self) -> IdAppareil {
        self.marque.a
    }
}

/// Motif de refus d'une écriture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Raison {
    /// Une entrée de `vu` n'est pas `< marque` (E-2, A-1).
    VuNonAnterieur,
    /// Module, genre, id ou nom de champ vide.
    IdentifiantVide,
    /// `marque.l ≥ L_MAX` (A-11).
    MarqueHorsBornes,
}

/// Refus d'un lot : première écriture fautive, dans l'ordre du lot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ErreurLot {
    pub index: usize,
    pub raison: Raison,
}

/// Valide un lot en entier (RG-ST01-4.5, INV-ST01-7).
///
/// Post : `Ok` ⇔ chaque écriture a des identifiants non vides, `l < L_MAX`
/// et chaque entrée de `vu` `< marque`. Sinon la première fautive.
/// Les entrées de `vu` en trop ne sont pas refusées (A-1).
pub fn valider(lot: &[Ecriture]) -> Result<(), ErreurLot> {
    for (index, w) in lot.iter().enumerate() {
        let raison = if !identifiants_pleins(&w.donnee, &w.quoi) {
            Raison::IdentifiantVide
        } else if w.marque.l >= L_MAX {
            Raison::MarqueHorsBornes
        } else if w.vu.values().any(|m| *m >= w.marque) {
            Raison::VuNonAnterieur
        } else {
            continue;
        };
        return Err(ErreurLot { index, raison });
    }
    Ok(())
}

/// Module, genre, id et nom de champ non vides.
pub(crate) fn identifiants_pleins(donnee: &IdDonnee, quoi: &Quoi) -> bool {
    let nom_plein = match quoi {
        Quoi::Champ { nom, .. } => !nom.0.is_empty(),
        Quoi::Suppression => true,
    };
    nom_plein && !donnee.module.is_empty() && !donnee.genre.is_empty() && !donnee.id.is_empty()
}
