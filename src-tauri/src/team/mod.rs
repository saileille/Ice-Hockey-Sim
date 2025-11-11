pub mod lineup;
pub mod ai;

use rand::{Rng, rngs::ThreadRng};
use serde_json::json;
use time::Date;
use crate::{
    competition::Competition, database::TEAMS, person::{Contract, manager::Manager, player::
        Player
    }, team::ai::PlayerNeed, time::date_to_db_string, types::{
        CompetitionId, ManagerId, PlayerId, TeamId, convert
    }
};
use self::lineup::LineUp;

#[derive(Debug)]
#[derive(Default, Clone)]
pub struct Team {
    pub id: TeamId,
    pub name: String,
    pub roster: Vec<PlayerId>,
    pub manager_id: ManagerId,
    pub lineup: LineUp,
    pub primary_comp_id: CompetitionId,

    // Player-acquisition related.
    pub approached_players: Vec<PlayerId>,
    pub player_needs: Vec<PlayerNeed>,

    // Actions.
    pub actions_remaining: u8
}

// Basics.
impl Team {
    fn build(name: &str) -> Self {
        Self {
            id: convert::int::<usize, TeamId>(TEAMS.lock().unwrap().len() + 1),
            name: name.to_string(),
            ..Default::default()
        }
    }

    // Create a team and store it in the database. Return a clone of the Team.
    pub fn build_and_save(name: &str) -> Self {
        let team = Self::build(name);
        team.save();
        return team;
    }

    pub fn fetch_from_db(id: &TeamId) -> Self {
        TEAMS.lock().unwrap().get(id)
            .expect(&format!("no Team with id {id:#?}")).clone()
    }

    // Update the Team to database.
    pub fn save(&self) {
        TEAMS.lock().unwrap().insert(self.id, self.clone());
    }

    // Delete the Team from the database.
    pub fn delete_from_db(&self) {
        TEAMS.lock().unwrap().remove(&self.id);
    }

    // Get the team's manager.
    pub fn get_manager(&self) -> Option<Manager> {
        Manager::fetch_from_db(&self.manager_id)
    }

    // Get every player in the roster.
    fn get_players(&self) -> Vec<Player> {
        self.roster.iter().map(|id| {
            let player = Player::fetch_from_db(id).unwrap();
            if !player.person.is_active {
                println!("{} has a retired player {} ({})", self.name, player.person.get_full_name(), player.id);
            }
            player

        }).collect()
    }

    // Get the players to whom the team has offered contracts.
    fn get_approached_players(&self) -> Vec<Player> {
        self.approached_players.iter().map(|id| Player::fetch_from_db(id).unwrap()).collect()
    }

    fn get_primary_competition(&self) -> Competition {
        Competition::fetch_from_db(&self.primary_comp_id)
    }

    // Get info for a team screen in JSON.
    pub fn get_team_screen_package(&self, today: &Date) -> serde_json::Value {
        let mut players = self.get_players();
        let mut approached_players = self.get_approached_players();
        players.append(&mut approached_players);

        players.sort_by(|a, b| (a.position_id.clone() as u8).cmp(&(b.position_id.clone() as u8)).then(b.ability.get_display().cmp(&a.ability.get_display())));

        let json_players: Vec<serde_json::Value> = players.iter().map(|a| a.get_package(today)).collect();
        json!({
            "id": self.id,
            "name": self.name,
            "manager": match self.get_manager() {
                Some(manager) => Some(manager.get_team_screen_json()),
                _ => None
            },
            "players": json_players
        })
    }

    // Get relevant team info for a contract.
    pub fn get_contract_package(&self) -> serde_json::Value {
        json!({
            "id": self.id,
            "name": self.name
        })
    }

    // Get relevant info of the team for a human manager.
    pub fn get_manager_package(&self) -> serde_json::Value {
        json!({
            "id": self.id,
            "actions_remaining": self.actions_remaining,
            "roster_overview": self.get_roster_overview_package(),
            "approached_players": self.approached_players,
        })
    }

    // Get a roster overview of the team.
    fn get_roster_overview_package(&self) -> Vec<serde_json::Value> {
        let mut overview: Vec<serde_json::Value> = self.roster.iter().map(|id| {
            let player = Player::fetch_from_db(id).unwrap();
            player.get_roster_overview_package(true)
        }).collect();

        let mut approached = self.approached_players.iter().map(|id| {
            let player = Player::fetch_from_db(id).unwrap();
            player.get_roster_overview_package(false)
        }).collect();

        overview.append(&mut approached);
        return overview;
    }
}

impl Team {
    // Build a lineup for the team from its roster.
    pub fn auto_build_lineup(&mut self) {
        self.lineup.clear();

        let mut players = self.get_players();
        players.sort_by(|a, b| b.ability.get_display().cmp(&a.ability.get_display()));

        self.lineup.auto_add(players);
        self.save();
    }

    // Give the team its full actions back.
    // Action value could depend on quantity and quality of team staff?
    pub fn return_actions_to_full(&mut self) {
        self.actions_remaining = 1;
    }

    // Return whether this day is the season end date.
    fn is_season_end_date(&self, today: &Date) -> bool {
        return self.get_primary_competition().season_window.is_last_day(today);
    }
}

impl Team {
    // Create a manager out of thin air.
    fn create_manager(&mut self, today: &Date, rng: &mut ThreadRng) {
        let mut manager = Manager::build_and_save_random(today, rng);
        self.manager_id = manager.id;
        manager.person.contract = Some(Contract::build(&date_to_db_string(today), &date_to_db_string(&Date::MAX), self.id));
        manager.save();
    }

    // Set up the team when initialising a game.
    pub fn setup(&mut self, today: &Date, rng: &mut ThreadRng) {
        self.create_manager(today, rng);
        self.return_actions_to_full();
        self.promote_junior_players(today, rng);
        self.save();
    }

    // Give a few junior players to the team at the end of the season.
    fn promote_junior_players(&mut self, today: &Date, rng: &mut ThreadRng) {
        for _ in 0..rng.random_range(1..=3) {
            let mut player = Player::create(today, rng, 16, 19);
            let contract = Contract::build_from_years(self, today, 4);
            player.person.contract = Some(contract);
            self.roster.push(player.id);
            player.save();
        }
    }

    pub fn season_end_checker(&mut self, today: &Date, rng: &mut ThreadRng) {
        if self.is_season_end_date(today) {
            self.promote_junior_players(today, rng);
        }
    }
}