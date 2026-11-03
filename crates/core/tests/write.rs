//! Écritures et validation de lot (T-4). Matrice US-4, cas 11 ; A-1, A-11.
//! Cas 12 et 13 (auteur hors liste, appareil ≠ marque.a) : non représentables, écart E-1 de l'archi.

mod support;

use arbitre_core::*;
use support::*;

fn valide() -> Ecriture {
    let mut w = ecriture(donnee("d"), champ("f", "v"), marque(10, 0, 1));
    w.vu.insert(visee("f"), marque(9, 0, 2));
    w.vu.insert(Visee::Suppression, marque(5, 3, 1));
    w
}

fn refus(w: Ecriture) -> Result<(), ErreurLot> {
    valider(&[valide(), w])
}

#[test]
fn us4_cas11_ecriture_complete_de_chaque_auteur() {
    let auteurs = [
        Auteur::UtilisateurCli,
        Auteur::UtilisateurInterface,
        Auteur::Assistant,
        Auteur::Systeme,
    ];
    let lot: Vec<Ecriture> = auteurs
        .iter()
        .map(|&auteur| Ecriture { auteur, ..valide() })
        .collect();
    assert_eq!(valider(&lot), Ok(()));
    let suppression = Ecriture {
        quoi: Quoi::Suppression,
        ..valide()
    };
    assert_eq!(valider(&[suppression]), Ok(()));
}

#[test]
fn lot_vide_valide() {
    assert_eq!(valider(&[]), Ok(()));
}

#[test]
fn cible_et_appareil() {
    let w = valide();
    assert_eq!(
        w.cible(),
        Cible {
            donnee: donnee("d"),
            visee: visee("f")
        }
    );
    assert_eq!(w.appareil(), app(1));
    let s = Ecriture {
        quoi: Quoi::Suppression,
        ..valide()
    };
    assert_eq!(
        s.cible(),
        Cible {
            donnee: donnee("d"),
            visee: Visee::Suppression
        }
    );
}

#[test]
fn vu_egal_a_la_marque_refuse() {
    let mut w = valide();
    w.vu.insert(visee("f"), w.marque);
    assert_eq!(
        refus(w),
        Err(ErreurLot {
            index: 1,
            raison: Raison::VuNonAnterieur
        })
    );
}

#[test]
fn vu_superieur_a_la_marque_refuse() {
    let mut w = valide();
    w.vu.insert(visee("f"), marque(11, 0, 0));
    assert_eq!(
        refus(w),
        Err(ErreurLot {
            index: 1,
            raison: Raison::VuNonAnterieur
        })
    );
}

#[test]
fn seule_la_deuxieme_entree_de_vu_fautive() {
    let mut w = valide();
    // Suppression < Champ : la première entrée (suppression) est valide, la seconde (champ) fautive.
    w.vu.insert(visee("f"), marque(10, 0, 2));
    assert_eq!(
        refus(w),
        Err(ErreurLot {
            index: 1,
            raison: Raison::VuNonAnterieur
        })
    );
}

#[test]
fn vu_en_trop_sur_un_autre_champ_accepte() {
    let mut w = valide();
    w.vu.insert(visee("autre"), marque(1, 0, 3));
    assert_eq!(valider(&[w]), Ok(()));
}

#[test]
fn identifiants_vides_refuses() {
    let vides = [
        Ecriture {
            donnee: IdDonnee {
                module: String::new(),
                ..donnee("d")
            },
            ..valide()
        },
        Ecriture {
            donnee: IdDonnee {
                genre: String::new(),
                ..donnee("d")
            },
            ..valide()
        },
        Ecriture {
            donnee: donnee(""),
            ..valide()
        },
        Ecriture {
            quoi: champ("", "v"),
            ..valide()
        },
    ];
    for w in vides {
        assert_eq!(
            refus(w),
            Err(ErreurLot {
                index: 1,
                raison: Raison::IdentifiantVide
            })
        );
    }
}

#[test]
fn valeur_vide_acceptee() {
    assert_eq!(
        valider(&[Ecriture {
            quoi: champ("f", ""),
            ..valide()
        }]),
        Ok(())
    );
}

#[test]
fn marque_l_max_refusee() {
    let w = Ecriture {
        marque: marque(L_MAX, 0, 1),
        ..valide()
    };
    assert_eq!(
        refus(w),
        Err(ErreurLot {
            index: 1,
            raison: Raison::MarqueHorsBornes
        })
    );
}

#[test]
fn marque_l_max_moins_un_acceptee() {
    assert_eq!(
        valider(&[Ecriture {
            marque: marque(L_MAX - 1, 0, 1),
            ..valide()
        }]),
        Ok(())
    );
}

#[test]
fn index_de_la_premiere_fautive() {
    let fautive = Ecriture {
        donnee: donnee(""),
        ..valide()
    };
    let lot = [
        valide(),
        valide(),
        fautive.clone(),
        Ecriture {
            marque: marque(L_MAX, 0, 1),
            ..valide()
        },
        fautive,
    ];
    assert_eq!(
        valider(&lot),
        Err(ErreurLot {
            index: 2,
            raison: Raison::IdentifiantVide
        })
    );
}
