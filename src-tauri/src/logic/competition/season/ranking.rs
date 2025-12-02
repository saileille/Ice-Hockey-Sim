// Functions and methods for ranking teams within a season.
use std::{cmp::Ordering, collections::HashMap};

use rand::{rngs::ThreadRng, seq::IndexedRandom};
use serde::{Deserialize, Serialize};

use crate::logic::{competition::{Competition, round_robin::RoundRobin as RoundRobinFormat, season::{Season, team::TeamSeason}}, types::Db};

// What ranking criteria a competition has.
#[derive(Debug, Serialize, Deserialize)]
#[derive(Eq, Hash, PartialEq)]
#[derive(Clone)]
pub enum RankCriteria {
    Seed,   // Lower is better.
    Points,
    GoalDifference,
    GoalsScored,
    GoalsConceded,  // Lower is better.
    RegularWins,
    TotalWins,
    OvertimeWins,
    Draws,
    OvertimeLosses,
    RegularLosses,  // Lower is better.
    TotalLosses,    // Lower is better.

    // Takes rankings from all child competitions, with latest competition having highest priority.
    ChildCompRanking,

    // Usually last resort, although competitions should have the ability to not sort at all.
    Random,
}

type CmpFunc = fn (&TeamSeason, &TeamSeason, &Option<RoundRobinFormat>, &mut ThreadRng) -> Ordering;

// Compare functions here.

fn compare_seed(a: &TeamSeason, b: &TeamSeason, _rr: &Option<RoundRobinFormat>, _rng: &mut ThreadRng) -> Ordering {
    a.seed.cmp(&b.seed)
}

fn compare_points(a: &TeamSeason, b: &TeamSeason, rr: &Option<RoundRobinFormat>, _rng: &mut ThreadRng) -> Ordering {
    b.points(rr).cmp(&a.points(rr))
}

fn compare_goal_difference(a: &TeamSeason, b: &TeamSeason, _rr: &Option<RoundRobinFormat>, _rng: &mut ThreadRng) -> Ordering {
    b.goal_difference.cmp(&a.goal_difference)
}

fn compare_goals_scored(a: &TeamSeason, b: &TeamSeason, _rr: &Option<RoundRobinFormat>, _rng: &mut ThreadRng) -> Ordering {
    b.goals_scored.cmp(&a.goals_scored)
}

fn compare_goals_conceded(a: &TeamSeason, b: &TeamSeason, _rr: &Option<RoundRobinFormat>, _rng: &mut ThreadRng) -> Ordering {
    a.goals_conceded.cmp(&b.goals_conceded)
}

fn compare_regular_wins(a: &TeamSeason, b: &TeamSeason, _rr: &Option<RoundRobinFormat>, _rng: &mut ThreadRng) -> Ordering {
    b.regular_wins.cmp(&a.regular_wins)
}

fn compare_total_wins(a: &TeamSeason, b: &TeamSeason, _rr: &Option<RoundRobinFormat>, _rng: &mut ThreadRng) -> Ordering {
    b.all_wins.cmp(&a.all_wins)
}

fn compare_overtime_wins(a: &TeamSeason, b: &TeamSeason, _rr: &Option<RoundRobinFormat>, _rng: &mut ThreadRng) -> Ordering {
    b.ot_wins.cmp(&a.ot_wins)
}

fn compare_draws(a: &TeamSeason, b: &TeamSeason, _rr: &Option<RoundRobinFormat>, _rng: &mut ThreadRng) -> Ordering {
    b.draws.cmp(&a.draws)
}

fn compare_overtime_losses(a: &TeamSeason, b: &TeamSeason, _rr: &Option<RoundRobinFormat>, _rng: &mut ThreadRng) -> Ordering {
    b.ot_losses.cmp(&a.ot_losses)
}

fn compare_regular_losses(a: &TeamSeason, b: &TeamSeason, _rr: &Option<RoundRobinFormat>, _rng: &mut ThreadRng) -> Ordering {
    a.regular_losses.cmp(&b.regular_losses)
}

fn compare_total_losses(a: &TeamSeason, b: &TeamSeason, _rr: &Option<RoundRobinFormat>, _rng: &mut ThreadRng) -> Ordering {
    a.all_losses.cmp(&b.all_losses)
}

fn compare_child_comp_ranking(_a: &TeamSeason, _b: &TeamSeason, _rr: &Option<RoundRobinFormat>, _rng: &mut ThreadRng) -> Ordering {
    // TODO... maybe
    Ordering::Equal
}

fn compare_random(_a: &TeamSeason, _b: &TeamSeason, _rr: &Option<RoundRobinFormat>, rng: &mut ThreadRng) -> Ordering {
    *[Ordering::Greater, Ordering::Less].choose(rng).unwrap()
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
    // Return a boolean for whether any sorting was done.
    pub async fn rank_teams(&self, db: &Db, comp: &Competition) -> bool {
        let mut teams = self.teams(db).await;
        if self.round_robin.is_some() {
            comp.sort_some_teams(db, &mut teams).await;
            self.save_rank(db, teams).await;
            return true;
        }
        else if self.knockout_round.is_some() {
            return self.sort_knockout_round(db, comp, &mut teams).await;
        }

        // Parent competition stuff here...
        // For now, it only does ChildCompRanking.
        else {
            // Using Box::pin to avoid recursion problems with async.
            return Box::pin(self.sort_child_competitions(db, comp, &mut teams)).await;
        }
    }

    // Sort a knockout round.
    async fn sort_knockout_round(&self, db: &Db, comp: &Competition, teams: &mut Vec<TeamSeason>) -> bool {
        let mut sorted_teams = self.knockout_round.as_ref().unwrap().advanced_teams.clone();
        let mut eliminated_teams = self.knockout_round.as_ref().unwrap().eliminated_teams.clone();

        comp.sort_some_teams(db, &mut sorted_teams).await;
        comp.sort_some_teams(db, &mut eliminated_teams).await;

        sorted_teams.append(&mut eliminated_teams);
        if sorted_teams.len() >= teams.len() {
            self.save_rank(db, sorted_teams).await;
            return true;
        }
        return false;
    }

    // Sort child competitions and determine the ranking based on them.
    async fn sort_child_competitions(&self, db: &Db, comp: &Competition, teams: &mut Vec<TeamSeason>) -> bool {
        let mut ranks = Vec::new();
        let child_comps = comp.children(db).await;

        for child_comp in child_comps {
            let season = child_comp.current_season(db).await;

            let sorted = season.rank_teams(db, &child_comp).await;
            if sorted {
                ranks.push(season.teams(db).await);
            }
        }

        let mut team_ranking: Vec<TeamSeason> = Vec::new();
        for rank in ranks.into_iter().rev() {
            for team in rank {
                let mut is_added = false;
                for ranked_team in team_ranking.iter() {
                    if team.team_id == ranked_team.team_id {
                        is_added = true;
                        break;
                    }
                }

                if !is_added {
                    team_ranking.push(team);
                }
            }
        }

        if team_ranking.len() >= teams.len() {
            self.save_rank(db, team_ranking).await;
            return true;
        }
        return false;
    }
}