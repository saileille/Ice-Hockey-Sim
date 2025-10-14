// Seasons represent a single iteration of a particular competition.
pub mod team;
pub mod round_robin;
pub mod knockout;
pub mod ranking;
mod schedule_generator;

use serde_json::json;
use time::Date;

use crate::{competition::{season::{self, team::TeamCompData}, Competition}, database::SEASONS, match_event::Game, time::{date_to_db_string, db_string_to_date}, types::{convert, CompetitionId, TeamId}};

#[derive(Debug, serde::Serialize)]
#[derive(Default, Clone)]
pub struct Season {
    index: usize,   // For easier saving of the season.
    comp_id: CompetitionId,
    name: String,   // Years during which the season takes place.
    pub teams: Vec<TeamCompData>,
    start_date: String,
    pub end_date: String,
    pub round_robin: Option<season::round_robin::RoundRobin>,
    pub knockout: Option<season::knockout::KnockoutRound>,

    pub upcoming_games: Vec<Game>,  // Upcoming games are stored with earliest LAST.
    played_games: Vec<Game>,    // Played games are stored with earliest FIRST.

    // Helper for easily checking if the season is over.
    pub is_over: bool,
}

impl Season {
    // Build an element.
    fn build(comp: &Competition, teams: &Vec<TeamId>) -> Self {
        let mut season = Self::default();
        season.comp_id = comp.id;
        season.teams = teams.iter().map(|id | TeamCompData::build(*id, 0)).collect();

        let start_date = comp.season_window.get_next_start_date();
        let end_date = comp.season_window.get_next_end_date();

        season.start_date = date_to_db_string(&start_date);
        season.end_date = date_to_db_string(&end_date);

        season.name = if start_date.year() == end_date.year() {
            start_date.year().to_string()
        }
        else {
            format!("{}-{}", start_date.year(), end_date.year())
        };


        let format = &comp.format.as_ref();
        if format.is_none() { return season; }

        if format.unwrap().round_robin.as_ref().is_some() {
            season.round_robin = Some(season::round_robin::RoundRobin::build());
        }
        else if format.unwrap().knockout.as_ref().is_some() {
            season.knockout = Some(season::knockout::KnockoutRound::build());
        }
        else {
            panic!("{}\nformat: {:#?}", comp.name, comp.format);
        }

        return season;
    }

    // Build a season and save it to the database.
    // Also build seasons for all possible child competitions.
    pub fn build_and_save(comp: &Competition, teams: &Vec<TeamId>) -> Self {
        let mut season = Self::build(comp, teams);

        season.save_new();
        return season;
    }

    pub fn fetch_from_db(comp_id: &CompetitionId, index: usize) -> Self {
        SEASONS.lock().unwrap().get(comp_id)
            .expect(&format!("no Competition with id {comp_id}"))[index].clone()
    }

    // Save a season to the database for the first time.
    fn save_new(&mut self) {
        self.index = SEASONS.lock().unwrap().get(&self.comp_id)
            .expect(&format!("no Competition with id {}", self.comp_id)).len();

        SEASONS.lock().unwrap().get_mut(&self.comp_id).unwrap().push(self.clone());
    }

    // Update the Season to database.
    pub fn save(&self) {
        SEASONS.lock().unwrap().get_mut(&self.comp_id)
            .expect(&format!("no Competition with id {}", self.comp_id))[self.index] = self.clone();
    }

    // Get the competition of the season.
    fn get_competition(&self) -> Competition {
        Competition::fetch_from_db(&self.comp_id).unwrap()
    }

    // Get the full name of the season, with all parent competition names included.
    fn get_full_name(&self) -> String {
        let comp_name = self.get_competition().get_full_name(None);
        format!("{} {}", comp_name, self.name)
    }

    // Get some nice JSON for a competition screen.
    pub fn get_comp_screen_json(&self, comp: &Competition) -> serde_json::Value {
        let teams: Vec<serde_json::Value> = self.teams.iter().map(|a| a.get_json(comp)).collect();
        let upcoming_games: Vec<serde_json::Value> = self.upcoming_games.iter().map(|a| a.get_comp_screen_json()).collect();
        let played_games: Vec<serde_json::Value> = self.played_games.iter().map(|a| a.get_comp_screen_json()).collect();

        json!({
            "name": self.name,
            "teams": teams,
            "upcoming_games": upcoming_games,
            "played_games": played_games
        })
    }

    // Check if the season has enough teams to begin.
    // min_no_of_teams must be the competition's min_no_of_teams field.
    pub fn has_enough_teams(&self, min_no_of_teams: u8) -> bool {
        convert::usize_to_u8(self.teams.len()) >= min_no_of_teams
    }

    // Finalise the creation of a season for a particular competition.
    pub fn setup(&mut self, comp: &Competition) {
        // The order of the teams becomes correct by reversing.
        self.teams.reverse();

        if self.round_robin.is_some() {
            self.setup_round_robin(comp);
        }
        else if self.knockout.is_some() {
            self.setup_knockout(comp);
        }

        // In this case the competition must have child competitions, so set them up instead.
        else {
            let mut teams = Vec::new();
            for (i, id) in comp.child_comp_ids.iter().enumerate() {
                if i == 0 {
                    // Set up all the teams here if the child competition is the first one.
                    // Teams that cannot be added will go to the next rounds.
                    // Does not support group competitions yet.
                    teams = self.teams.clone();
                }
                Competition::fetch_from_db(id).unwrap().setup_season(&mut teams);
            }
        }
    }

    // Set up a round robin season.
    fn setup_round_robin(&mut self, comp: &Competition) {
        self.generate_schedule(comp);
    }

    // Set up a knockout season.
    fn setup_knockout(&mut self, comp: &Competition) {
        let teams = &self.teams;
        let start = &self.start_date;
        let end = &self.end_date;

        self.upcoming_games = self.knockout.as_mut().unwrap().setup(teams, start, end, comp);
    }

    // Update the teamdata to this season and all parent competition seasons.
    pub fn update_teamdata(&mut self, comp: &Competition, games: &Vec<Game>) {
        for team in self.teams.iter_mut() {
            for game in games.iter() {
                if team.team_id == game.home.team_id {
                    team.update(&game.home, &game.away, game.has_overtime());
                }
                else if team.team_id == game.away.team_id {
                    team.update(&game.away, &game.home, game.has_overtime());
                }
            }
        }

        // In case this is a knockout round, we need to update the pairs as well.
        if self.knockout.is_some() {
            self.knockout.as_mut().unwrap().update_teamdata(games);
        }

        self.rank_teams(comp);

        // We are not saving the season here, because we are doing it after updating the played_games vector.
        // self.save();

        // Update all parent competitions as well.
        if comp.parent_comp_id != 0 {
            let parent_comp = Competition::fetch_from_db(&comp.parent_comp_id).unwrap();

            let mut season = Season::fetch_from_db(&parent_comp.id, parent_comp.get_seasons_amount() - 1);
            season.update_teamdata(&parent_comp, games);
            season.save();
        }
    }

    // Get the team's index in self.teams.
    fn get_team_index(&self, id: TeamId) -> usize {
        self.teams.iter().position(|a | a.team_id == id).unwrap()
    }

    // Get all games of this season (not including sub or parent competitions).
    pub fn get_all_games(&self) -> Vec<Game> {
        let mut games: Vec<Game> = self.upcoming_games.iter().cloned().collect();
        games.append(&mut self.played_games.clone());
        return games;
    }

    // Simulate the games for this day.
    pub fn simulate_day(&mut self, comp: &Competition, today: &Date) {
        let mut games = Vec::new();

        while !self.upcoming_games.is_empty() {
            let mut game = self.upcoming_games.swap_remove(self.upcoming_games.len() - 1);

            // Play the game if it happens today.
            if db_string_to_date(&game.date) == *today {
                game.play();
                games.push(game);
            }

            // Otherwise return the game back to the upcoming games and exit the loop.
            else {
                self.upcoming_games.push(game);
                break;
            }
        }

        if games.is_empty() { return; }

        self.update_teamdata(comp, &games);
        self.played_games.append(&mut games);

        self.check_if_over(comp);
        self.save();
    }

    // Check if the season has ended, and react appropriately.
    // Return whether over or not.
    fn check_if_over(&mut self, comp: &Competition) -> bool {
        // No need to do more.
        if self.is_over { return true; }

        if self.round_robin.is_some() {
            if !self.upcoming_games.is_empty() { return false; }
            self.is_over = true;
            self.do_post_season_tasks_rr(comp);
        }

        if self.knockout.is_some() {
            if self.knockout.as_mut().unwrap().check_if_over(comp, &mut self.upcoming_games) {
                self.is_over = true;
                self.do_post_season_tasks_ko(comp);
            }
        }

        return true;
    }

    // Do round robin post-season tasks.
    fn do_post_season_tasks_rr(&mut self, comp: &Competition) {
        self.rank_teams(comp);

        for connection in comp.connections.iter() {
            connection.send_teams(&self.teams);
        }
    }

    // Do knockout post-season tasks.
    fn do_post_season_tasks_ko(&mut self, comp: &Competition) {
        for connection in comp.connections.iter() {
            // Needs sorting, still.
            connection.send_teams(&self.knockout.as_ref().unwrap().advanced_teams);
        }
    }
}

// Tests.
impl Season {
    // Display a match schedule of all matches of this competition's season, and child seasons.
    pub fn display_match_schedule(&self, comp: &Competition) -> String {
        let mut games = Vec::new();
        games.append(&mut self.get_all_games().clone());

        for id in comp.child_comp_ids.iter() {
            let season = Season::fetch_from_db(id, self.index);
            games.append(&mut season.get_all_games().clone())
        }

        games.sort_by(|a, b|
            db_string_to_date(&a.date).cmp(&db_string_to_date(&b.date))
            .then(a.get_name().cmp(&b.get_name())));

        let mut s = String::new();
        let mut current_date = Date::MIN;
        for game in games {
            let game_date = db_string_to_date(&game.date);
            if current_date < db_string_to_date(&game.date) {
                current_date = game_date;

                if !s.is_empty() {
                    s += "\n\n";
                }

                s += &format!("{}", date_to_db_string(&current_date));
            }

            s += &format!("\n{}", game.get_name_and_score_if_started());
        }

        return s;
    }
}