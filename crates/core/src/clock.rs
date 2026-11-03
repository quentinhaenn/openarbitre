//! Port `Clock` et son simulateur (archi ST-01 § Interfaces, T-3).

/// Source d'heure physique, en millisecondes UTC depuis l'époque Unix (RG-ST01-4.4).
pub trait Clock {
    fn now(&self) -> u64;
}

/// Source d'heure commandable, une par appareil simulé (PROP-ST01-1).
///
/// Contrat (à confirmer, voir `06_skeleton.md`) : `now()` rend la dernière heure posée ;
/// après `figer()`, `avancer` et `reculer` sont sans effet jusqu'au prochain `fixer`.
#[cfg(feature = "sim")]
#[derive(Debug, Default)]
pub struct HorlogeSimulee {
    t: core::cell::Cell<u64>,
    figee: core::cell::Cell<bool>,
}

#[cfg(feature = "sim")]
impl HorlogeSimulee {
    /// Source posée sur `t`, non figée.
    pub fn nouvelle(t: u64) -> Self {
        Self {
            t: t.into(),
            figee: false.into(),
        }
    }

    /// Pose l'heure à `t` et lève le gel.
    pub fn fixer(&self, t: u64) {
        self.t.set(t);
        self.figee.set(false);
    }

    /// Avance de `delta` ms (sans effet si figée).
    pub fn avancer(&self, delta: u64) {
        if !self.figee.get() {
            self.t.set(self.t.get().saturating_add(delta));
        }
    }

    /// Recule de `delta` ms, borné à 0 (sans effet si figée).
    pub fn reculer(&self, delta: u64) {
        if !self.figee.get() {
            self.t.set(self.t.get().saturating_sub(delta));
        }
    }

    /// Gèle l'heure courante.
    pub fn figer(&self) {
        self.figee.set(true);
    }
}

#[cfg(feature = "sim")]
impl Clock for HorlogeSimulee {
    fn now(&self) -> u64 {
        self.t.get()
    }
}
