// Functions and methods for ranking teams within a season.

use std::{cmp::Ordering, collections::HashMap};

use rand::{rng, seq::IndexedRandom};

use crate::{competition::{format, season::{team::TeamCompData, Season}, Competition}, team::Team};

// What ranking criteria a competition has.
#[derive(Debug)]
#[derive(Eq, Hash, PartialEq)]
#[derive(Clone)]
pub enum RankCriteria {
    Seed,
    Points,
    GoalDifference,
    GoalsScored,
    GoalsConceded,
    RegularWins,
    TotalWins,
    OvertimeWins,
    Draws,
    OvertimeLosses,
    RegularLosses,
    TotalLosses,

    // Takes rankings from all child competitions, with latest competition having highest priority.
    ChildCompRanking,

    // Usually last resort, although competitions should have the ability to not sort at all.
    Random,
}

type CmpFunc = fn (&TeamCompData, &TeamCompData, &Option<format::round_robin::RoundRobin>) -> Ordering;

// Compare functions here.

fn compare_seed(a: &TeamCompData, b: &TeamCompData, _rr: &Option<format::round_robin::RoundRobin>) -> Ordering {
    a.seed.cmp(&b.seed)
}

fn compare_points(a: &TeamCompData, b: &TeamCompData, rr: &Option<format::round_robin::RoundRobin>) -> Ordering {
    b.get_points(rr).cmp(&a.get_points(rr))
}

fn compare_goal_difference(a: &TeamCompData, b: &TeamCompData, _rr: &Option<format::round_robin::RoundRobin>) -> Ordering {
    b.get_goal_difference().cmp(&a.get_goal_difference())
}

fn compare_goals_scored(a: &TeamCompData, b: &TeamCompData, _rr: &Option<format::round_robin::RoundRobin>) -> Ordering {
    b.goals_scored.cmp(&a.goals_scored)
}

fn compare_goals_conceded(a: &TeamCompData, b: &TeamCompData, _rr: &Option<format::round_robin::RoundRobin>) -> Ordering {
    a.goals_conceded.cmp(&b.goals_conceded)
}

fn compare_regular_wins(a: &TeamCompData, b: &TeamCompData, _rr: &Option<format::round_robin::RoundRobin>) -> Ordering {
    b.regular_wins.cmp(&a.regular_wins)
}

fn compare_total_wins(a: &TeamCompData, b: &TeamCompData, _rr: &Option<format::round_robin::RoundRobin>) -> Ordering {
    b.get_wins().cmp(&a.get_wins())
}

fn compare_overtime_wins(a: &TeamCompData, b: &TeamCompData, _rr: &Option<format::round_robin::RoundRobin>) -> Ordering {
    b.ot_wins.cmp(&a.ot_wins)
}

fn compare_draws(a: &TeamCompData, b: &TeamCompData, _rr: &Option<format::round_robin::RoundRobin>) -> Ordering {
    b.draws.cmp(&a.draws)
}

fn compare_overtime_losses(a: &TeamCompData, b: &TeamCompData, _rr: &Option<format::round_robin::RoundRobin>) -> Ordering {
    b.ot_losses.cmp(&a.ot_losses)
}

fn compare_regular_losses(a: &TeamCompData, b: &TeamCompData, _rr: &Option<format::round_robin::RoundRobin>) -> Ordering {
    a.regular_losses.cmp(&b.regular_losses)
}

fn compare_total_losses(a: &TeamCompData, b: &TeamCompData, _rr: &Option<format::round_robin::RoundRobin>) -> Ordering {
    a.get_losses().cmp(&b.get_losses())
}

fn compare_child_comp_ranking(_a: &TeamCompData, _b: &TeamCompData, _rr: &Option<format::round_robin::RoundRobin>) -> Ordering {
    // TODO... maybe
    Ordering::Equal
}

fn compare_random(_a: &TeamCompData, _b: &TeamCompData, _rr: &Option<format::round_robin::RoundRobin>) -> Ordering {
    let mut rng = rng();
    *[Ordering::Greater, Ordering::Less].choose(&mut rng).unwrap()
}

// Get the available sort functions.
pub fn get_sort_functions() -> HashMap<RankCriteria, CmpFunc> {
    let mut functions: HashMap<RankCriteria, CmpFunc> = HashMap::new();
    functions.insert(RankCriteria::Seed, compare_seed);
    functions.insert(RankCriteria::Points, compare_points);
    functions.insert(RankCriteria::GoalDifference, compare_goal_difference);
    functions.insert(RankCriteria::GoalsScored, compare_goals_scored);
    functions.insert(RankCriteria::GoalsConceded, compare_goals_conceded);
    functions.insert(RankCriteria::RegularWins, compare_regular_wins);
    functions.insert(RankCriteria::TotalWins, compare_total_wins);
    functions.insert(RankCriteria::OvertimeWins, compare_overtime_wins);
    functions.insert(RankCriteria::Draws, compare_draws);
    functions.insert(RankCriteria::OvertimeLosses, compare_overtime_losses);
    functions.insert(RankCriteria::RegularLosses, compare_regular_losses);
    functions.insert(RankCriteria::TotalLosses, compare_total_losses);
    functions.insert(RankCriteria::ChildCompRanking, compare_child_comp_ranking);
    functions.insert(RankCriteria::Random,compare_random);
    return functions;
}

impl Season {
    // Get the teams in the order of betterhood.
    pub fn rank_teams(&mut self, comp: &Competition) {
        if self.round_robin.is_some() {
            comp.sort_some_teams(&mut self.teams);
        }
        else if self.knockout.is_some() {
            self.sort_knockout_round(comp);
        }

        // Parent competition stuff here...
        // For now, it only does ChildCompRanking.
        else {
            self.sort_parent_competition(comp);
        }
    }

    // Sorting a knockout round is a bit more complex..
    fn sort_knockout_round(&mut self, comp: &Competition) {
        let mut advanced_teams = self.knockout.as_ref().unwrap().advanced_teams.clone();
        let mut eliminated_teams = self.knockout.as_ref().unwrap().eliminated_teams.clone();

        comp.sort_some_teams(&mut advanced_teams);
        comp.sort_some_teams(&mut eliminated_teams);

        advanced_teams.append(&mut eliminated_teams);
        if advanced_teams.len() >= self.teams.len() {
            self.teams = advanced_teams;
        }
    }

    // Sorting a parent competition is bound to be even more complex...
    fn sort_parent_competition(&mut self, comp: &Competition) {
        let mut ranks = Vec::new();
        for id in comp.child_comp_ids.iter() {
            let comp = Competition::fetch_from_db(id).unwrap();
            let mut season = Season::fetch_from_db(id, self.index);

            season.rank_teams(&comp);
            ranks.push(season.teams.clone());
        }

        let mut team_ranking: Vec<TeamCompData> = Vec::new();
        for rank in ranks.iter().rev() {
            for team in rank.iter() {
                let mut is_added = false;
                for ranked_team in team_ranking.iter() {
                    if team.team_id == ranked_team.team_id {
                        is_added = true;
                        break;
                    }
                }

                if !is_added {
                    team_ranking.push(team.clone());
                }
            }
        }

        if team_ranking.len() >= self.teams.len() {
            self.teams = team_ranking;
        }
    }
}

impl Season {
    // Get a final ranking for the season.
    pub fn display_final_ranking(&mut self) -> String {
        let comp = Competition::fetch_from_db(&self.comp_id).unwrap();
        self.rank_teams(&comp);

        let mut s = String::new();
        for (i, team) in self.teams.iter().enumerate() {
            if i != 0 { s += "\n"; }
            s += &format!("{}.\t{}", i + 1, Team::fetch_from_db(&team.team_id).name);
        }

        return s;
    }
}