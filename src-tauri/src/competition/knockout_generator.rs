// Functions for generating knockout competitions.

use ordinal::ToOrdinal as _;
use rand::{Rng, rngs::ThreadRng};
use time::{Date, Duration};

use crate::{competition::{self, CompConnection, Competition, Seed, format::{self, knockout_round::KnockoutRound as KnockoutRoundFormat}, season::ranking::RankCriteria}, database::COMPETITIONS, match_event, time::{AnnualDate, AnnualWindow, get_dates}, types::{CompetitionId, convert}};

// Generate a knockout competition with each round being represented as its own competition element.
pub fn build(
    name: &str, // Name of the knockout competition itself.
    round_names: Vec<&str>, // Names for the rounds to be generated. If there are more rounds than names, the remaining round will have automatically generated names.
    season_window: AnnualWindow, // Time when this competition is played.
    match_rules: Vec<match_event::Rules>,    // Match rules for each round. If there are more rounds than elements in this vector, the last element will be applied to remaining rounds.
    wins_required: Vec<u8>, // Wins required to advance from each round. If there are more rounds than elements in this vector, the last element will be applied to remaining rounds.
    mut teams_in_rounds: Vec<u8>,   // Number of teams the knockout competition has on each round.
    teams_at_end: u8,   // Number of teams the knockout competition ends with.
    connections: Vec<CompConnection>,    // Connections to other competitions; where to move which teams after the knockout is over.
    rank_criteria: Vec<RankCriteria>, today: &Date, rng: &mut ThreadRng
) {
    let mut parent_comp = Competition::build_and_save(name, Vec::new(), season_window, connections, teams_in_rounds[0], None, Vec::new(), Vec::new(), today);
    parent_comp.competition_type = competition::Type::Tournament;

    get_teams_in_rounds(&mut teams_in_rounds, teams_at_end);
    let mut rounds = create_rounds(round_names, match_rules, wins_required, teams_in_rounds, rank_criteria, parent_comp.id);

    set_date_boundaries(&mut rounds, &parent_comp.season_window, rng);
    finalise_rounds(&mut parent_comp, &mut rounds, today);
}

// Get how many teams each round should actually have.
fn get_teams_in_rounds(teams_in_rounds: &mut Vec<u8>, teams_at_end: u8) {
    let mut last_rounds = Vec::new();
    let mut next_teams = teams_at_end * 2;
    while next_teams < *teams_in_rounds.last().unwrap() {
        last_rounds.push(next_teams);
        next_teams *= 2;
    }

    // Return the no of teams for each round here if there is no need for special rounds.
    if &next_teams == teams_in_rounds.last().unwrap() {
        last_rounds.reverse();
        teams_in_rounds.append(&mut last_rounds);
        return;
    }

    // In case there is need for a special round...
    let start_teams = teams_in_rounds.swap_remove(teams_in_rounds.len() - 1);
    let round_size = (start_teams - last_rounds.last().unwrap()) * 2;

    teams_in_rounds.push(round_size);
    last_rounds.reverse();
    teams_in_rounds.append(&mut last_rounds);
}

// Create rounds.
fn create_rounds(round_names: Vec<&str>, match_rules: Vec<match_event::Rules>, wins_required: Vec<u8>, teams_in_rounds: Vec<u8>, rank_criteria: Vec<RankCriteria>, parent_comp_id: CompetitionId) -> Vec<Competition> {
    let mut rounds = Vec::new();

    for (i, round_size) in teams_in_rounds.iter().enumerate() {
        // Create the round and add as many values to it as I can.
        let mut round = Competition {
            id: convert::int::<usize, CompetitionId>(COMPETITIONS.lock().unwrap().len() + 1),
            parent_comp_id: parent_comp_id,
            min_no_of_teams: *round_size,
            rank_criteria: rank_criteria.clone(),
            format: format::Format::build(
                None,
                Some(KnockoutRoundFormat::build(
                    get_from_index_or_last(&wins_required, i)
            )),
            get_from_index_or_last(&match_rules, i)
            ),
            ..Default::default()
        };

        // Give the round a name from predefined options, or a default one.
        match i < round_names.len() {
            true => round.name = round_names[i].to_string(),
            _ => assign_default_name(&mut round, i, teams_in_rounds.len())
        };

        rounds.push(round);
    }

    return rounds;
}

// Get a generic name for a knockout round based on how many teams it has.
fn assign_default_name(round: &mut Competition, round_index: usize, total_rounds: usize) {
    let rounds_left = total_rounds - round_index;

    match (rounds_left, round.min_no_of_teams) {
        (1, 2) => round.name = "Final".to_string(),
        (2, 4) => round.name = "Semi Final".to_string(),
        (3, 8) => round.name = "Quarter Final".to_string(),
        _ => round.name = format!("{} Round", (round_index + 1).to_ordinal_string())
    };
}

// Give each round's games a proportionate time window.
fn set_date_boundaries(rounds: &mut Vec<Competition>, season_duration: &AnnualWindow, rng: &mut ThreadRng) {
    // Let's get our example dates from a year that was not a leap year.
    let (start_date, end_date) = season_duration.get_dates_from_start_year(1900);
    let available_days = get_dates(&start_date, &end_date);

    let round_durations = get_round_durations(rounds, convert::int::<usize, u8>(available_days.len()), rng);

    // Starting from one day backwards.
    let mut last_date = available_days[0].checked_sub(Duration::days(1)).unwrap();
    for (i, round) in rounds.iter_mut().enumerate() {
        let start = last_date.next_day().unwrap();
        last_date = last_date.checked_add(Duration::days(round_durations[i] as i64)).unwrap();
        let end = last_date.clone();

        //round.season_window = AnnualWindow::build(start.month() as u8, start.day(), end.month() as u8, end.day());
        round.season_window = AnnualWindow::build(
            AnnualDate::build(start.month(), start.day()),
            AnnualDate::build(end.month(), end.day())
        );
    }
}

// Get a duration for each round in the knockout stage.
fn get_round_durations(rounds: &[Competition], days: u8, rng: &mut ThreadRng) -> Vec<u8> {
    let matches_in_rounds: Vec<f64> = rounds.iter().map(|round| round.format.as_ref().unwrap().knockout_round.as_ref().unwrap().get_maximum_matches_in_pair() as f64).collect();
    let total_matches: f64 = matches_in_rounds.iter().sum();

    // Minimum amount of dates in each round.
    let mut round_durations: Vec<u8> = matches_in_rounds.iter().map(|a| (a / total_matches * (days as f64)) as u8).collect();

    // Calculating leftovers.
    let assigned_dates: u8 = round_durations.iter().sum();
    let mut leftover_dates = days - assigned_dates;

    // Assign the leftovers randomly.
    let mut round_indexes: Vec<usize> = (0..rounds.len()).collect();
    while leftover_dates > 0 {
        let index = round_indexes.swap_remove(rng.random_range(0..round_indexes.len()));
        round_durations[index] += 1;
        leftover_dates -= 1;
    }

    return round_durations;
}

// Get an element by index, or the last element in case index is too large.
fn get_from_index_or_last<T: Clone>(vector: &[T], index: usize) -> T {
    match index < vector.len() {
        true => vector[index].clone(),
        _ => vector.last().unwrap().clone()
    }
}

// Finalise the rounds.
fn finalise_rounds(parent_comp: &mut Competition, rounds: &mut Vec<Competition>, today: &Date) {
    let rounds_clone = rounds.clone();
    let mut connections = vec![0; rounds.len()];

    let teams = Vec::new();
    for (i, round) in rounds.iter_mut().enumerate() {
        round.save_new(&teams, today);
        parent_comp.child_comp_ids.push(round.id);

        if i == rounds_clone.len() - 1 { continue; }

        // Doing connections.
        create_connections(i.clone(), round, &rounds_clone, &mut connections);
        round.save();
    }

    parent_comp.save();
}

// Create connections between knockout rounds.
fn create_connections(i: usize, round: &mut Competition, rounds: &[Competition], connections: &mut Vec<u8>) {
    let mut advancing_teams = convert::int::<u8, i8>(round.min_no_of_teams / 2);

    let mut last;
    let mut first;
    let mut iteration = 0;
    while advancing_teams > 0 {
        iteration += 1;
        let space_in_next_round = rounds[i].min_no_of_teams - connections[i];

        last = advancing_teams;
        first = (advancing_teams - convert::int::<u8, i8>(space_in_next_round) + 1).clamp(1, i8::MAX);
        let added_teams = last - first + 1;

        round.connections.push(CompConnection::build([first as u8, last as u8], round.id + iteration, Seed::Preserve, false));
        connections[i] += added_teams as u8;
        advancing_teams -= added_teams;
    }
}