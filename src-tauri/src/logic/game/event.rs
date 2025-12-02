// An event is anything worth of writing down that happens during a match.
// Shot, goal, penalty, etc.
use rand::{rngs::ThreadRng, seq::IndexedRandom};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::logic::{self, event::Id, game::{Game, team::TeamGame}, person::player::Player, team::lineup::cache::PlayersOnIceCache, types::{GameEventId, GameId, GameSeconds, PersonId, TeamId}};

#[derive(Debug)]
#[derive(Default, Clone)]
#[derive(Serialize, Deserialize)]
pub struct PlayersOnIce {
    gk: PersonId,
    ld: PersonId,
    rd: PersonId,
    lw: PersonId,
    c: PersonId,
    rw: PersonId,
    extra_attacker: PersonId,
}

impl PlayersOnIce {
    pub fn build(gk: PersonId, ld: PersonId, rd: PersonId, lw: PersonId, c: PersonId, rw: PersonId, extra_attacker: PersonId) -> Self {
        Self { gk, ld, rd, lw, c, rw, extra_attacker, }
    }
}

#[derive(Debug)]
#[derive(Default, Clone)]
#[derive(Serialize, Deserialize)]
#[derive(FromRow)]
pub struct Event {
    pub id: GameEventId,
    pub game_id: GameId,
    pub target_team_id: TeamId,
    pub opponent_team_id: TeamId,
    pub time: GameSeconds,
    pub target_players: PlayersOnIce,
    pub opponent_players: PlayersOnIce,
}

impl Event {
    fn build(game: &Game, target_team: &TeamGame, opponent_team: &TeamGame) -> Self {
        Self {
            game_id: game.id,
            target_team_id: target_team.team_id,
            opponent_team_id: opponent_team.team_id,
            time: game.total_seconds(),
            target_players: target_team.lineup_cache.players_on_ice.ids(),
            opponent_players: opponent_team.lineup_cache.players_on_ice.ids(),

            ..Default::default()
        }
    }
}

#[derive(Debug)]
#[derive(Default, Clone)]
#[derive(Serialize, Deserialize)]
pub struct Shot {
    pub event: Event,
    pub shooter_id: PersonId,
    pub assister_1_id: PersonId,
    pub assister_2_id: PersonId,
    pub is_goal: bool,
}

// Basics.
impl Shot {
    fn build(game: &Game, attacker: &TeamGame, defender: &TeamGame) -> Self {
        Self {
            event: Event::build(game, attacker, defender),
            ..Default::default()
        }
    }

    // Do the building, calculating, simulating, everything, here.
    // pub fn simulate(rng: &mut ThreadRng, game_id: GameId, target_team_id: TeamId, opponent_team_id: TeamId, time: GameSeconds, target_players: &PlayersOnIceCache, opponent_players: &PlayersOnIceCache) -> Self {
    pub fn simulate(rng: &mut ThreadRng, game: &Game, attacker: &TeamGame, defender: &TeamGame) -> Self {
        let mut shot = Self::build(game, attacker, defender);

        let shooter_and_assisters = shot.create_shooter_and_assisters(&attacker.lineup_cache.players_on_ice, rng);
        shot.calculate_goal(&shooter_and_assisters, &defender.lineup_cache.players_on_ice, rng);

        return shot;
    }

    // Determine who shoots and who assists.
    // Completely random for now.
    fn create_shooter_and_assisters(&mut self, attackers: &PlayersOnIceCache, rng: &mut ThreadRng) -> Vec<Player> {
        let players = attackers.create_vector_of_skaters();
        let mut shooter_and_assisters_ids = Vec::new();

        let mut shooter_and_assisters = Vec::new();

        for i in 0..3 {
            let chosen = players.choose(rng)
                .expect(&format!("could not choose Player. iteration: {i}, players.len(): {}, players on ice: {attackers:#?}", players.len()));

            let id = chosen.person.id;


            if shooter_and_assisters_ids.contains(&id) {
                break;
            }
            else {
                shooter_and_assisters_ids.push(id);
                shooter_and_assisters.push(chosen.clone());
            }
        }

        for id in shooter_and_assisters_ids {
            if self.shooter_id == 0 {
                self.shooter_id = id;
            }
            else if self.assister_1_id == 0 {
                self.assister_1_id = id;
            }
            else {
                self.assister_2_id = id;
            }
        }

        return shooter_and_assisters;
    }

    // Check if the shot ends up in goal.
    // Only taking shooter into account for now.
    fn calculate_goal(&mut self, shooter_and_assisters: &[Player], defenders: &PlayersOnIceCache, rng: &mut ThreadRng) {
        let gk_ability = defenders.gk.as_ref().unwrap().ability.get() as f64;
        let shooter_ability = shooter_and_assisters[0].ability.get() as f64;
        let total_ability = gk_ability + shooter_ability;
        let modifier;

        if total_ability == 0.0 { modifier = 0.5 }
        else { modifier = shooter_ability / total_ability }

        if logic::event::Type::fetch_from_db(Id::Goal).get_outcome(rng, modifier) {
            self.is_goal = true;
        }
    }
}