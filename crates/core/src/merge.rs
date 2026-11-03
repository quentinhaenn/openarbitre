//! Journal, écriture locale, fusion, projection, écartées, conflits, anomalies
//! (archi ST-01 ; amendements A-1 à A-4, A-10 ; T-5, T-6).

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec::Vec;

use crate::hlc::{Horloge, HorlogeEpuisee, IdAppareil, Marque};
use crate::write::{Auteur, Champ, Cible, Ecriture, ErreurLot, IdDonnee, Quoi, Raison, Valeur, Vu};

/// Contenu d'une identité `(cible, marque)` : `(valeur?, vu, auteur)` (A-3).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Contenu {
    pub valeur: Option<Valeur>,
    pub vu: Vu,
    pub auteur: Auteur,
}

/// Refus d'une écriture locale (A-2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErreurEcriture {
    HorlogeEpuisee,
    Invalide(Raison),
}

impl From<HorlogeEpuisee> for ErreurEcriture {
    fn from(_: HorlogeEpuisee) -> Self {
        Self::HorlogeEpuisee
    }
}

/// Statut d'une donnée projetée ; une donnée supprimée porte sa suppression retenue (RG-2.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statut {
    Presente,
    Supprimee(Ecriture),
}

/// État projeté d'une donnée. Les gagnantes restent présentes, masquées, si elle est supprimée (AMB-ST01-1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EtatDonnee {
    pub statut: Statut,
    pub champs: BTreeMap<Champ, Ecriture>,
}

/// État projeté `R(S)`, partie état.
pub type Projection = BTreeMap<IdDonnee, EtatDonnee>;

/// Paire en conflit (RG-ST01-2.1). `champ` est celui de l'écriture de champ.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Conflit {
    pub donnee: IdDonnee,
    pub champ: Champ,
    pub retenue: Ecriture,
    pub ecartee: Ecriture,
}

/// Identité portée par au moins deux contenus (MANQUE-ST01-2, A-3).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Anomalie {
    pub cible: Cible,
    pub marque: Marque,
    pub contenus: BTreeSet<Contenu>,
}

/// Sorties incrémentales d'une fusion : différences après \ avant, triées (RG-2.2, A-4, A-10).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Rapport {
    pub nouvelles_ecartees: Vec<Ecriture>,
    pub nouveaux_conflits: Vec<Conflit>,
    pub nouvelles_anomalies: Vec<Anomalie>,
}

/// Journal connu `S` d'un appareil et son horloge (A-2, A-3). `S` croît par union ; rien n'en sort.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Journal {
    horloge: Horloge,
    ecritures: BTreeMap<(Cible, Marque), BTreeSet<Contenu>>,
}

impl Journal {
    /// Journal vide de l'appareil `appareil`.
    pub fn nouveau(appareil: IdAppareil) -> Self {
        let _ = appareil;
        todo!("T-5: Journal::nouveau — A-2")
    }

    /// Seul chemin d'une écriture locale (A-2).
    ///
    /// Pré : aucune. Valide les identifiants **avant** d'émettre (un refus ne consomme pas de marque).
    /// Post : `vu` = `vu_pour` (A-1), écriture insérée et rendue pour l'envoi.
    pub fn ecrire(
        &mut self,
        pt: u64,
        donnee: IdDonnee,
        quoi: Quoi,
        auteur: Auteur,
    ) -> Result<Ecriture, ErreurEcriture> {
        let _ = (pt, donnee, quoi, auteur, &self.horloge, &self.ecritures);
        todo!("T-5: Journal::ecrire — A-1, A-2")
    }

    /// Fusionne un lot reçu, tout ou rien (RG-ST01-4.5, INV-ST01-7).
    ///
    /// Lot invalide ⇒ `Err`, journal **et** horloge inchangés. Sinon insère, puis l'horloge
    /// adopte la plus grande marque du lot (RG-ST01-4.3). Rapport = différences (A-10).
    /// Accepte un lot construit à la main (`vu` non vérifiable, Q-6).
    pub fn fusionner(&mut self, lot: &[Ecriture]) -> Result<Rapport, ErreurLot> {
        let _ = lot;
        todo!("T-5/T-6: Journal::fusionner — RG-2.2, RG-ST01-4.3, A-2, A-10")
    }

    /// Toutes les écritures connues, `S`, dans l'ordre `(cible, marque, contenu)`.
    pub fn ecritures(&self) -> Vec<Ecriture> {
        todo!("T-5: Journal::ecritures — A-3")
    }

    /// État projeté (RG-2.1, RG-2.3) : `supprimée ⇔ t_s > t_f` ; gagnante = rang `(marque, cible, contenu)` max.
    pub fn projection(&self) -> Projection {
        todo!("T-5: Journal::projection — RG-2.1, RG-2.3, INV-4, INV-6")
    }

    /// `écartées(S) = S \ retenues(S)`, ordre `(cible, marque, contenu)` (RG-2.4, INV-7, A-4).
    pub fn ecartees(&self) -> Vec<Ecriture> {
        todo!("T-5: Journal::ecartees — RG-2.4, INV-7")
    }

    /// Identités à plusieurs contenus, triées par identité (A-3).
    pub fn anomalies(&self) -> Vec<Anomalie> {
        todo!("T-5: Journal::anomalies — A-3, MANQUE-ST01-2")
    }

    /// `conflits(S)` (RG-ST01-2.1), par intervalle (A-10), triés et dédoublonnés (A-4).
    pub fn conflits(&self) -> Vec<Conflit> {
        todo!("T-6: Journal::conflits — RG-ST01-2.1, RG-ST01-2.2, A-4, A-10")
    }

    /// Table `vu` d'une nouvelle écriture locale (A-1) : champ `f` ⇒ entrées `Champ(f)` et
    /// `Suppression` ; suppression ⇒ une entrée par visée connue de la donnée. Plus grande marque connue.
    #[allow(dead_code)]
    fn vu_pour(&self, donnee: &IdDonnee, quoi: &Quoi) -> Vu {
        let _ = (donnee, quoi);
        todo!("T-5: vu_pour — A-1")
    }
}
