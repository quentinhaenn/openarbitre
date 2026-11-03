//! Marques, horloge logique hybride, avances (archi ST-01 § Types du cœur, A-11 ; T-3).

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::num::NonZeroU64;

use crate::write::Ecriture;

/// Borne exclusive de `Marque::l` (A-11).
pub const L_MAX: u64 = 1 << 53;

/// Identifiant d'appareil, ordonné octet par octet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct IdAppareil(pub [u8; 16]);

impl IdAppareil {
    /// Construit l'identifiant depuis un entier ; l'ordre des entiers est conservé.
    pub fn depuis_u128(n: u128) -> Self {
        Self(n.to_be_bytes())
    }
}

/// Marque `(l, c, a)`. L'ordre des champs est contractuel : `Ord` lexicographique (RG-ST01-4.1, A-4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Marque {
    pub l: u64,
    pub c: u32,
    pub a: IdAppareil,
}

/// Plus de marque émissible : `c` saturé ou `l' ≥ L_MAX` (MANQUE-ST01-3, A-11).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HorlogeEpuisee;

/// État `(l, c)` de l'horloge d'un appareil.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Horloge {
    l: u64,
    c: u32,
    a: IdAppareil,
}

impl Horloge {
    /// Horloge de l'appareil `a`, état `(0, 0)`.
    pub fn nouvelle(a: IdAppareil) -> Self {
        Self { l: 0, c: 0, a }
    }

    /// Émet une marque (RG-ST01-4.2) : `l' = max(l, pt)`, `c' = c + 1` si `l' = l`, sinon 0.
    ///
    /// Post : marque > tout ce qui a été émis ou reçu ; `l' ≥ pt` (INV-ST01-5).
    /// Erreur : `c` saturé ou `l' ≥ L_MAX` ⇒ `HorlogeEpuisee`, état inchangé (A-11).
    pub fn emettre(&mut self, pt: u64) -> Result<Marque, HorlogeEpuisee> {
        let l = self.l.max(pt);
        let c = if l == self.l {
            self.c.checked_add(1).ok_or(HorlogeEpuisee)?
        } else {
            0
        };
        if l >= L_MAX {
            return Err(HorlogeEpuisee);
        }
        (self.l, self.c) = (l, c);
        Ok(Marque { l, c, a: self.a })
    }

    /// Adopte `max_lex((l, c), (m.l, m.c))` (RG-ST01-4.3). Post : toute émission suivante > `m` (INV-ST01-6).
    pub fn recevoir(&mut self, m: &Marque) {
        (self.l, self.c) = (self.l, self.c).max((m.l, m.c));
    }
}

/// Seuil d'avance `σ` en ms ; `σ = 0` n'est pas représentable (RG-ST01-4.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Seuil(pub NonZeroU64);

/// 60 min (RG-ST01-4.6).
pub const SEUIL_DEFAUT: Seuil = Seuil(NonZeroU64::new(3_600_000).unwrap());

/// Avance maximale d'un appareil dans un lot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Avance {
    pub appareil: IdAppareil,
    pub avance_ms: u64,
}

/// Appareils dont une écriture du lot dépasse `maintenant` de plus de `seuil` (RG-ST01-4.6).
///
/// Post : une entrée par appareil, avance maximale du lot, triée par appareil ;
/// avance égale au seuil non signalée (INV-ST01-8). Pure.
pub fn avances(lot: &[Ecriture], maintenant: u64, seuil: Seuil) -> Vec<Avance> {
    let mut max: BTreeMap<IdAppareil, u64> = BTreeMap::new();
    for w in lot {
        let avance = max.entry(w.appareil()).or_default();
        *avance = (*avance).max(w.marque.l.saturating_sub(maintenant));
    }
    max.into_iter()
        .filter(|&(_, avance_ms)| avance_ms > seuil.0.get())
        .map(|(appareil, avance_ms)| Avance {
            appareil,
            avance_ms,
        })
        .collect()
}
