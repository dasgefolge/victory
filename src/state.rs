use crate::{
    identities::Identity,
    roles::Role,
};

pub(crate) struct State {
    players: Vec<Player>,
    phase: Phase,
    /// whether somebody died this round (relevant for queen)
    mourning: bool,
    target: PlayerID,
    shielded: PlayerID,
    pending_action: Option<Action>,
}

impl State {
    fn next_input(&self) -> InputRequest {
        unimplemented!() //TODO
    }

    fn advance_game(&mut self, input: Input) {
        match input {
            Input::Action(action) => {
                self.pending_action = Some(action);
                if !unimplemented!() /*TODO Engel kann diese Aktion engeln */ {
                    self.advance_game(Input::AngelPass);
                }
            }
            Input::AngelPass => match self.pending_action.expect("Engel kann nichts engeln") {
                Action::VoteResult(opt_p) => {
                    if let Some(p) = opt_p {
                        self.players[p].alive = false;
                        self.mourning = true;
                    }
                    //TODO check/reset Queen victory
                    self.phase = Phase::Hunter;
                }
            },
            _ => unimplemented!() //TODO
        }
    }
}

enum Input {
    Attempt(),
    Action(Action),
    AngelPass,
    AngelVeto,
}

enum Action {
    VoteResult(Option<PlayerID>), //individual votes are front-end issue
}

trait Frontend<P> {
    fn give_input(&mut self, ir: InputRequest) -> Input;
}

type PlayerID = usize;

struct Player {
    role: Role,
    identity: Identity,
    alive: bool,
    bitten: bool,
    /// has action token (relevant for angel and active vampire)
    active: bool,
    /// for example which potions remain
    props: Props,
}

enum Props {} //TODO
