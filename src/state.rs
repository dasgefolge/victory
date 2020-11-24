use {
    std::{
        collections::{
            HashMap,
            HashSet,
        },
        hash::Hash,
        mem,
    },
    hashbag::HashBag,
    itertools::{
        EitherOrBoth,
        Itertools as _,
    },
    multiset::HashMultiSet,
    rand::prelude::*,
    smart_default::SmartDefault,
    crate::{
        identities::{
            Identity,
            Wincon,
        },
        roles::Role,
        util::Cycle as _,
        player::{
            Character,
            Player,
        }
    },
};

pub trait PlayerId: Eq + Hash + Clone {}

impl<T: Eq + Hash + Clone> PlayerId for T {}

#[derive(Debug, SmartDefault)]
pub struct State<P: PlayerId> {
    points: HashMap<P, u32>,
    pub(crate) players: Vec<Player<P>>,
    just_quit: HashSet<P>,
    /// `None` means between segments (i.e. players can join/leave)
    pub(crate) phase: Option<Role>,
    /// whether somebody died this round (relevant for queen)
    pub(crate) mourning: bool,
    /// the target chosen by the hunter
    target: Option<Seat>, // TODO multiple hunters
    /// the person shielded from being vetoed _again_ by the angel
    shielded: Option<Seat>,
    /// `true` if the angel shield is from this cycle, `false` if it's from last
    shield_is_current: bool,
    /// the action pending approval by the angel (within frame set by front-end)
    pending_action: Option<Action>,
}

impl<P: PlayerId> State<P> {
    pub fn points(&self) -> &HashMap<P, u32> { &self.points }
    pub fn players(&self) -> &[Player<P>] { &self.players }
    pub fn just_quit(&self) -> &HashSet<P> { &self.just_quit }

    pub fn next_input(&self) -> InputRequest {
        if let Some(phase) = self.phase {
            if let Some(ref p_a) = self.pending_action {
                InputRequest::Angel(phase, p_a)
            } else {
                InputRequest::Action(phase)
            }
        } else {
            InputRequest::Meta
        }
    }

    /// # Panics
    ///
    /// Panics on invalid inputs (e.g. `AngelVeto` with a player ID that doesn't have a pending action).
    pub fn advance_game(&mut self, input: Input<P>) {
        match input {
            Input::Meta(meta_input) => {
                assert!(self.phase.is_none());
                match meta_input {
                    MetaInput::Join(p, join_seat) => {
                        assert!(!self.players.iter().any(|player| player.id == p));
                        assert!(!self.just_quit.iter().any(|player_id| *player_id == p));
                        let join_seat = join_seat.unwrap_or_else(|| thread_rng().gen_range(0, self.players.len().max(1)));
                        self.players.insert(join_seat, Player::new(p));
                        for player in &mut self.players {
                            if let Some(vampire_seat) = player.character.as_mut().and_then(|c| c.bitten_by.as_mut()) {
                                if *vampire_seat >= join_seat { *vampire_seat += 1 }
                            }
                        }
                    }
                    MetaInput::Quit(p) => {
                        let (quit_seat, _) = self.players.iter().enumerate().find(|(_, player)| player.id == p).expect("player is already not in the game");
                        self.players.retain(|player| player.id != p);
                        self.just_quit.insert(p);
                        self.cleanup_player_references(quit_seat);
                        for player in &mut self.players {
                            if let Some(vampire_seat) = player.character.as_mut().and_then(|c| c.bitten_by.as_mut()) {
                                if *vampire_seat > quit_seat { *vampire_seat -= 1 }
                            }
                        }
                    }
                    MetaInput::Go => {
                        fn free_attributes<P, T: Copy + Eq + Hash>(players: &[Player<P>], list: impl Iterator<Item = T>, player_attr: impl Fn(&Character) -> T) -> Vec<T> {
                            let attrs = list.collect::<HashMultiSet<_>>()
                                - players.iter()
                                    .filter_map(|player| player.character.as_ref())
                                    .map(player_attr)
                                    .collect();
                            let mut attrs = attrs.iter().copied().collect_vec();
                            attrs.shuffle(&mut thread_rng());
                            attrs
                        }
                        
                        self.just_quit.clear();
                        let free_roles = free_attributes(&self.players, Role::list(), |c| c.role);
                        let free_identities = free_attributes(&self.players, Identity::list(), |c| c.identity);
                        self.players.iter_mut()
                            .filter(|player| player.character.is_none())
                            .zip_longest(free_roles.into_iter().zip(free_identities))
                            .for_each(|pair| match pair {
                                EitherOrBoth::Left(_) => panic!("not enough roles and/or identities"),
                                EitherOrBoth::Right(_) => {}
                                EitherOrBoth::Both(dead_player, (free_role, free_identity)) => dead_player.character = Some(Character::new(free_role, free_identity)),
                            });
                        self.phase = Some(Role::default());
                    }
                }
            }
            Input::Ingame(ingame_input) => {
                assert!(self.phase.is_some());
                match ingame_input {
                    IngameInput::Action(action) => {
                        assert!(self.pending_action.is_none());
                        let actor_seats = if let Action::Eat(ref votes) = action { votes.keys().copied().collect() } else { self.current_actor_seats() };
                        assert!(actor_seats.iter().all(|&seat| self.players[seat].can_act()));
                        actor_seats.iter().for_each(|&seat| if let Some(ref mut c) = self.players[seat].character { c.ready = false });
                        if self.living_players_with_role(Role::Angel).next().is_some()
                        && actor_seats.iter().any(|&seat| self.shielded.map_or(true, |shielded| seat != shielded)) {
                            self.pending_action = Some(action);
                        } else {
                            self.resolve_action(actor_seats, action);
                        }
                    }
                    IngameInput::AngelPass => {
                        let actor_seats = self.current_actor_seats();
                        let pending = self.pending_action.take().expect("no pending action to pass");
                        self.resolve_action(actor_seats, pending);
                        self.pending_action = None;
                    }
                    IngameInput::AngelVeto(seat) => {
                        //TODO validate seat?
                        self.pending_action = None; //TODO Werwölfe?
                        self.shielded = Some(seat);
                        self.shield_is_current = true;
                        self.set_next_phase();
                    },
                    IngameInput::WinAttempt() => unimplemented!() //TODO
                }
            }
        }
    }

    fn living_players_with_role(&self, role: Role) -> impl Iterator<Item = Seat> + '_ {
        self.players.iter()
            .enumerate()
            .filter(move |(_, player)| player.character.as_ref().map_or(false, |c| c.role == role))
            .map(|(seat, _)| seat)
    }

    fn current_actor_seats(&self) -> Vec<Seat> {
        let phase = self.phase.expect("can't use actions in between segments");
        self.living_players_with_role(phase)
            .map(|seat| if let Some(vampire_seat) = self.players[seat].character.as_ref().expect("living_players_with_role yielded a dead player").bitten_by { vampire_seat } else { seat })
            .collect()
    }

    fn resolve_action(&mut self, actor_seats: Vec<Seat>, action: Action) {
        match action {
            Action::Shoot => self.kill(self.target.expect("no target to shoot")),
            Action::Aim(seat) => {
                assert!(self.players[seat].character.is_some()); // can't aim at dead players
                self.target = Some(seat);
            }
            Action::Bite(seat) => {
                if let Some(ref mut c) = self.players[seat].character {
                    c.bitten_by = Some(actor_seats.into_iter().exactly_one().expect("unclear who bit you"));
                } else {
                    panic!("can't bite dead player")
                }
            }
            Action::Swap(seat0, seat1) => {
                //TODO
            }
            Action::Eat(map) => {
                let counts = map.into_iter().map(|(_, target)| target).collect::<HashBag<_>>();
                if let Some((seat, _)) = counts.into_iter().max_by_key(|(_, count)| *count) {
                    self.kill(seat);
                }
            },
            Action::VoteResult(opt_p) => if let Some(p) = opt_p { self.kill(p) },
        }
        self.set_next_phase();
    }

    fn set_next_phase(&mut self) {
        let mut phase = self.phase.expect("can't advance phase in between segments");
        loop {
            phase = phase.succ();
            self.check_wincons();
            if phase == Role::Angel {
                // angel phase used for end-of-cycle cleanup
                if self.shield_is_current {
                    self.shield_is_current = false;
                } else {
                    self.shielded = None;
                }
                self.mourning = false;
                self.target = None;
            } else if !self.living_players_with_role(phase).any(|seat| self.players[seat].can_act() /*TODO and has choices to make */) { break }
        }
        self.phase = Some(phase);
    }

    fn check_wincons(&mut self) {
        let vs = self.players.iter()
            .enumerate()
            .filter(|(_, player)| player.character.as_ref().map_or(false,
                |c| if let Wincon::Static(check) = c.identity.wincon() { check(self) } else { false }
            ))
            .map(|(seat, _)| seat)
            .collect_vec();
        if !vs.is_empty() { self.victory(vs) }
    }

    fn kill(&mut self, seat: Seat) { //TODO day/night? method?
        assert!(self.players[seat].character.is_some()); // make sure player was alive
        self.players[seat].character = None;
        self.mourning = true;
        if self.target == Some(seat) { self.target = None } //TODO move to cleanup_player_references? (depends on how kicking players/handling disconnects work)
        self.cleanup_player_references(seat);
    }

    /// Called when a living player dies or leaves the game.
    fn cleanup_player_references(&mut self, seat: Seat) {
        for player in &mut self.players {
            if let Some(ref mut c) = player.character {
                if let Some(vampire_seat) = c.bitten_by {
                    if vampire_seat == seat { c.bitten_by = None }
                }
            }
        }
    }

    fn victory(&mut self, mut victors: Vec<Seat>) {
        let with_victors = self.players.iter()
            .enumerate()
            .filter(|(_, player)| player.character.as_ref().map_or(false, |c| c.identity.wincon().is_with(self, &victors)))
            .map(|(seat, _)| seat)
            .collect_vec();
        victors.extend(with_victors);
        let State { mut points, mut players, .. } = mem::replace(self, State::default());
        victors.iter().for_each(|&seat| players[seat].character = None); // “kill” all victors so they get a new role/identity and count for the number of points
        let gain = players.iter().filter(|player| player.character.is_none()).count() as u32;
        victors.iter().for_each(|&seat| *points.entry(players[seat].id.clone()).or_default() += gain);
        self.points = points;
        self.players = players;
    }

    pub(crate) fn player_by_id(&self, pid: &P) -> &Player<P> {
        self.players.iter().find(|player| player.id == *pid).expect("no such player")
    }
}

pub enum InputRequest<'a> {
    /// Accepting all [`Input::Meta`].
    Meta,
    /// Accepting actions or win attempts.
    Action(Role),
    /// Accepting angel vetos, angel passes, or win attempts.
    Angel(Role, &'a Action),
}

pub enum Input<P: Eq + Hash> {
    Meta(MetaInput<P>),
    Ingame(IngameInput),
}
pub enum MetaInput<P: Eq + Hash> {
    Join(P, Option<Seat>),
    Quit(P),
    Go,
}

pub enum IngameInput {
    Action(Action),
    AngelPass,
    AngelVeto(Seat),
    WinAttempt(),
}

#[derive(Debug)]
pub enum Action {
    VoteResult(Option<Seat>), //individual votes are front-end issue
    Shoot,
    Aim(Seat),
    Bite(Seat),
    Swap(Seat, Seat),
    Eat(HashMap<Seat, Seat>),
}

pub(crate) type Seat = usize;
