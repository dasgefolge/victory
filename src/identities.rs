use {
    std::convert::Infallible as Never,
    crate::state::State,
    self::{
        Identity::*,
        Party::*,
        SoloIdentity::*,
        Wincon::*,
    },
};

#[derive(Clone, Copy)]
enum SoloIdentity {
    Sherlock,
    V,
    JackTheRipper,
    Queen,
    Macbeth,
}

pub(crate) enum Identity {
    Solo(SoloIdentity),
    Romeo,
    Juliet,
    Single,
    Churchill,
    Doyle,
    Mozart,
}

impl Identity {
    fn party(&self) -> Party {
        match *self {
            Solo(solo) => SoloParty(solo),
            Romeo | Juliet | Single => Lovers,
            Churchill | Doyle | Mozart => Masons,
        }
    }

    fn wincon(&self, state: &State) -> Wincon {
        let mason_wincon = Flips(unimplemented!()); //TODO
        match self {
            Churchill => Either(Box::new(mason_wincon), Box::new(With(SoloParty(Queen)))),
            Doyle => Either(Box::new(mason_wincon), Box::new(With(SoloParty(Sherlock)))),
            Mozart => Either(Box::new(mason_wincon), Box::new(With(Lovers))),
            _ => unimplemented!(), //TODO
        }
    }
}

type Flip = Never; // "IO Bool"

enum Wincon {
    Static(Box<dyn Fn(&State) -> bool>),
    Flips(Flip),
    With(Party),
    Either(Box<Wincon>, Box<Wincon>),
}

enum Party {
    SoloParty(SoloIdentity),
    Lovers,
    Masons,
}
