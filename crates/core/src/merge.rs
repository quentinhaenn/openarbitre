//! Journal, écriture locale, fusion, projection, écartées, conflits, anomalies
//! (archi ST-01 ; amendements A-1 à A-4, A-10 ; T-5, T-6).

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec::Vec;
use core::ops::Bound;

use crate::hlc::{Horloge, HorlogeEpuisee, IdAppareil, Marque};
use crate::write::{
    Auteur, Champ, Cible, Ecriture, ErreurLot, IdDonnee, Quoi, Raison, Valeur, Visee, Vu,
    identifiants_pleins, valider,
};

const MARQUE_MIN: Marque = Marque {
    l: 0,
    c: 0,
    a: IdAppareil([0; 16]),
};

/// Rang d'une écriture `(marque, cible, contenu)` (A-3).
fn rang(w: &Ecriture) -> (Marque, Cible, Option<&Valeur>, &Vu, Auteur) {
    let valeur = match &w.quoi {
        Quoi::Champ { valeur, .. } => Some(valeur),
        Quoi::Suppression => None,
    };
    (w.marque, w.cible(), valeur, &w.vu, w.auteur)
}

/// Clé de tri des conflits (A-4).
fn cle_conflit(k: &Conflit) -> impl Ord + '_ {
    (
        &k.donnee,
        &k.champ,
        k.retenue.marque,
        k.ecartee.marque,
        rang(&k.retenue),
        rang(&k.ecartee),
    )
}

fn en_ecriture(cible: &Cible, marque: &Marque, contenu: &Contenu) -> Ecriture {
    let quoi = match (&cible.visee, &contenu.valeur) {
        (Visee::Champ(nom), Some(valeur)) => Quoi::Champ {
            nom: nom.clone(),
            valeur: valeur.clone(),
        },
        _ => Quoi::Suppression,
    };
    Ecriture {
        donnee: cible.donnee.clone(),
        quoi,
        marque: *marque,
        vu: contenu.vu.clone(),
        auteur: contenu.auteur,
    }
}

/// Éléments de `apres` absents de `avant`, ordre de `apres` conservé.
fn nouveaux<T: Ord>(avant: Vec<T>, apres: Vec<T>) -> Vec<T> {
    let avant: BTreeSet<T> = avant.into_iter().collect();
    apres.into_iter().filter(|x| !avant.contains(x)).collect()
}

/// `w` est retenue dans l'état projeté de sa donnée.
fn retenue(etat: &EtatDonnee, w: &Ecriture) -> bool {
    match &w.quoi {
        Quoi::Champ { nom, .. } => etat.champs.get(nom) == Some(w),
        Quoi::Suppression => matches!(&etat.statut, Statut::Supprimee(s) if s == w),
    }
}

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
        Self {
            horloge: Horloge::nouvelle(appareil),
            ecritures: BTreeMap::new(),
        }
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
        if !identifiants_pleins(&donnee, &quoi) {
            return Err(ErreurEcriture::Invalide(Raison::IdentifiantVide));
        }
        let marque = self.horloge.emettre(pt)?;
        let vu = self.vu_pour(&donnee, &quoi);
        let w = Ecriture {
            donnee,
            quoi,
            marque,
            vu,
            auteur,
        };
        self.inserer(&w);
        Ok(w)
    }

    /// Fusionne un lot reçu, tout ou rien (RG-ST01-4.5, INV-ST01-7).
    ///
    /// Lot invalide ⇒ `Err`, journal **et** horloge inchangés. Sinon insère, puis l'horloge
    /// adopte la plus grande marque du lot (RG-ST01-4.3). Rapport = différences (A-10).
    /// Accepte un lot construit à la main (`vu` non vérifiable, Q-6).
    pub fn fusionner(&mut self, lot: &[Ecriture]) -> Result<Rapport, ErreurLot> {
        valider(lot)?;
        // ponytail: recalcul par donnée touchée, quadratique sur un flux d'écritures isolées — voir ST-19
        let touchees: BTreeSet<&IdDonnee> = lot.iter().map(|w| &w.donnee).collect();
        let avant: Vec<_> = touchees.iter().map(|d| self.sorties(d)).collect();
        for w in lot {
            self.inserer(w);
        }
        if let Some(m) = lot.iter().map(|w| w.marque).max() {
            self.horloge.recevoir(&m);
        }
        let mut rapport = Rapport::default();
        for (d, (ecartees, conflits, anomalies)) in touchees.iter().zip(avant) {
            let (ecartees2, conflits2, anomalies2) = self.sorties(d);
            rapport
                .nouvelles_ecartees
                .extend(nouveaux(ecartees, ecartees2));
            rapport
                .nouveaux_conflits
                .extend(nouveaux(conflits, conflits2));
            rapport
                .nouvelles_anomalies
                .extend(nouveaux(anomalies, anomalies2));
        }
        Ok(rapport)
    }

    /// Toutes les écritures connues, `S`, dans l'ordre `(cible, marque, contenu)`.
    pub fn ecritures(&self) -> Vec<Ecriture> {
        self.ecritures
            .iter()
            .flat_map(|((c, m), ks)| ks.iter().map(|k| en_ecriture(c, m, k)))
            .collect()
    }

    /// État projeté (RG-2.1, RG-2.3) : `supprimée ⇔ t_s > t_f` ; gagnante = rang `(marque, cible, contenu)` max.
    pub fn projection(&self) -> Projection {
        self.donnees()
            .into_iter()
            .map(|d| (d.clone(), self.projeter(&d)))
            .collect()
    }

    /// `écartées(S) = S \ retenues(S)`, ordre `(cible, marque, contenu)` (RG-2.4, INV-7, A-4).
    pub fn ecartees(&self) -> Vec<Ecriture> {
        self.donnees()
            .iter()
            .flat_map(|d| self.ecartees_de(d))
            .collect()
    }

    /// Identités à plusieurs contenus, triées par identité (A-3).
    pub fn anomalies(&self) -> Vec<Anomalie> {
        self.donnees()
            .iter()
            .flat_map(|d| self.anomalies_de(d))
            .collect()
    }

    /// `conflits(S)` (RG-ST01-2.1), par intervalle (A-10), triés et dédoublonnés (A-4).
    pub fn conflits(&self) -> Vec<Conflit> {
        self.donnees()
            .iter()
            .flat_map(|d| self.conflits_de(d))
            .collect()
    }

    /// Table `vu` d'une nouvelle écriture locale (A-1) : champ `f` ⇒ entrées `Champ(f)` et
    /// `Suppression` ; suppression ⇒ une entrée par visée connue de la donnée. Plus grande marque connue.
    fn vu_pour(&self, donnee: &IdDonnee, quoi: &Quoi) -> Vu {
        let mut vu = Vu::new();
        for ((c, m), _) in self.de(donnee) {
            let utile = match quoi {
                Quoi::Suppression => true,
                Quoi::Champ { nom, .. } => {
                    c.visee == Visee::Suppression || c.visee == Visee::Champ(nom.clone())
                }
            };
            if utile {
                vu.insert(c.visee.clone(), *m);
            }
        }
        vu
    }

    fn inserer(&mut self, w: &Ecriture) {
        let valeur = match &w.quoi {
            Quoi::Champ { valeur, .. } => Some(valeur.clone()),
            Quoi::Suppression => None,
        };
        let contenu = Contenu {
            valeur,
            vu: w.vu.clone(),
            auteur: w.auteur,
        };
        self.ecritures
            .entry((w.cible(), w.marque))
            .or_default()
            .insert(contenu);
    }

    /// Entrées d'une donnée, dans l'ordre `(cible, marque)`.
    fn de(&self, d: &IdDonnee) -> impl Iterator<Item = (&(Cible, Marque), &BTreeSet<Contenu>)> {
        let debut = (
            Cible {
                donnee: d.clone(),
                visee: Visee::Suppression,
            },
            MARQUE_MIN,
        );
        self.ecritures
            .range(debut..)
            .take_while(move |((c, _), _)| c.donnee == *d)
    }

    fn donnees(&self) -> Vec<IdDonnee> {
        let mut v: Vec<IdDonnee> = Vec::new();
        for (c, _) in self.ecritures.keys() {
            if v.last() != Some(&c.donnee) {
                v.push(c.donnee.clone());
            }
        }
        v
    }

    fn sorties(&self, d: &IdDonnee) -> (Vec<Ecriture>, Vec<Conflit>, Vec<Anomalie>) {
        (
            self.ecartees_de(d),
            self.conflits_de(d),
            self.anomalies_de(d),
        )
    }

    fn projeter(&self, d: &IdDonnee) -> EtatDonnee {
        let mut champs = BTreeMap::new();
        let mut suppression: Option<Ecriture> = None;
        // l'ordre `(cible, marque, contenu)` met la gagnante en dernier
        for ((c, m), ks) in self.de(d) {
            for k in ks {
                let w = en_ecriture(c, m, k);
                match &c.visee {
                    Visee::Champ(nom) => {
                        champs.insert(nom.clone(), w);
                    }
                    Visee::Suppression => suppression = Some(w),
                }
            }
        }
        let t_f = champs.values().map(|w| w.marque).max();
        let statut = match suppression {
            Some(s) if Some(s.marque) > t_f => Statut::Supprimee(s),
            _ => Statut::Presente,
        };
        EtatDonnee { statut, champs }
    }

    fn ecartees_de(&self, d: &IdDonnee) -> Vec<Ecriture> {
        let etat = self.projeter(d);
        let toutes = self
            .de(d)
            .flat_map(|((c, m), ks)| ks.iter().map(|k| en_ecriture(c, m, k)));
        toutes.filter(|w| !retenue(&etat, w)).collect()
    }

    fn anomalies_de(&self, d: &IdDonnee) -> Vec<Anomalie> {
        let collisions = self.de(d).filter(|(_, ks)| ks.len() > 1);
        collisions
            .map(|((c, m), ks)| Anomalie {
                cible: c.clone(),
                marque: *m,
                contenus: ks.clone(),
            })
            .collect()
    }

    fn conflits_de(&self, d: &IdDonnee) -> Vec<Conflit> {
        let mut visees_champ: Vec<Cible> = Vec::new();
        for ((c, _), _) in self.de(d) {
            if matches!(c.visee, Visee::Champ(_)) && visees_champ.last() != Some(c) {
                visees_champ.push(c.clone());
            }
        }
        let suppression = Cible {
            donnee: d.clone(),
            visee: Visee::Suppression,
        };
        let mut conflits = Vec::new();
        for ((cw, mw), contenus) in self.de(d) {
            for kw in contenus {
                let w = en_ecriture(cw, mw, kw);
                let propres;
                let cibles: &[Cible] = match cw.visee {
                    Visee::Suppression => &visees_champ,
                    Visee::Champ(_) => {
                        propres = [cw.clone(), suppression.clone()];
                        &propres
                    }
                };
                for c in cibles {
                    // intervalle `]vu(w)[c], marque(w)]` : les écritures de `c` que l'auteur de `w` n'avait pas vues
                    let bas = match kw.vu.get(&c.visee) {
                        Some(m) => Bound::Excluded((c.clone(), *m)),
                        None => Bound::Included((c.clone(), MARQUE_MIN)),
                    };
                    for ((cx, mx), ksx) in self
                        .ecritures
                        .range((bas, Bound::Included((c.clone(), *mw))))
                    {
                        for kx in ksx {
                            let x = en_ecriture(cx, mx, kx);
                            let a_vu = |w: &Ecriture, x: &Ecriture| {
                                w.vu.get(&x.cible().visee).is_some_and(|m| *m >= x.marque)
                            };
                            if x == w || kx.valeur == kw.valeur || a_vu(&x, &w) {
                                continue;
                            }
                            let ((Visee::Champ(nom), _) | (_, Visee::Champ(nom))) =
                                (&cw.visee, &c.visee)
                            else {
                                continue;
                            };
                            let champ = nom.clone();
                            let (retenue, ecartee) = if rang(&w) > rang(&x) {
                                (w.clone(), x)
                            } else {
                                (x, w.clone())
                            };
                            conflits.push(Conflit {
                                donnee: d.clone(),
                                champ,
                                retenue,
                                ecartee,
                            });
                        }
                    }
                }
            }
        }
        conflits.sort_by(|a, b| cle_conflit(a).cmp(&cle_conflit(b)));
        conflits.dedup();
        conflits
    }
}
