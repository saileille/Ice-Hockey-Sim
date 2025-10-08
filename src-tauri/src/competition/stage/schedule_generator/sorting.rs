// Functions and methods for sorting and comparing teams when generating matches.
use std::{
    collections::HashMap,
    cmp::Ordering
};
use rand::{
    rngs::ThreadRng,
    seq::SliceRandom
};

use crate::{
    competition::stage::rules::MatchGenType, types::{convert, TeamId}
};

use super::match_generator::TeamScheduleData;

// The type that pieces of sort functions use.
type CmpFunc = fn (&TeamScheduleData, &TeamScheduleData, &TeamScheduleData, &TeamScheduleData) -> Ordering;

// Compare the home-away difference. Team with more need for a home game comes first.
fn compare_home_away(a: &TeamScheduleData, b: &TeamScheduleData, prev_a: &TeamScheduleData, prev_b: &TeamScheduleData) -> Ordering {
    let a_diff: i8 = a.get_home_away_difference(prev_a);
    let b_diff: i8 = b.get_home_away_difference(prev_b);

    return b_diff.cmp(&a_diff);
}

// Compare the home-away difference. Team with more need for an away game comes first.
fn compare_away_home(a: &TeamScheduleData, b: &TeamScheduleData, prev_a: &TeamScheduleData, prev_b: &TeamScheduleData) -> Ordering {
    return compare_home_away(a, b, prev_a, prev_b).reverse();
}

// Compare the absolute value of home-away difference Team with more need for either home or away game comes first.
fn compare_home_away_abs(a: &TeamScheduleData, b: &TeamScheduleData, prev_a: &TeamScheduleData, prev_b: &TeamScheduleData) -> Ordering {
    let a_diff: i8 = a.get_home_away_difference(prev_a).abs();
    let b_diff: i8 = b.get_home_away_difference(prev_b).abs();

    return b_diff.cmp(&a_diff);
}

// Compare the match count.
fn compare_match_count(a: &TeamScheduleData, b: &TeamScheduleData, prev_a: &TeamScheduleData, prev_b: &TeamScheduleData) -> Ordering {
    let a_total: i8 = convert::u8_to_i8(a.get_match_count(prev_a));
    let b_total: i8 = convert::u8_to_i8(b.get_match_count(prev_b));

    return a_total.cmp(&b_total);
}

// Get the previous schedule datas for compare functions.
fn get_previous(a: &TeamScheduleData, b: &TeamScheduleData, prev_schedule_map: &HashMap<TeamId, TeamScheduleData>) -> (TeamScheduleData, TeamScheduleData) {
    let prev_a: TeamScheduleData = match prev_schedule_map.get(&a.team_id) {
        Some(prev) => prev.clone(),
        None => TeamScheduleData::default(),
    };

    let prev_b: TeamScheduleData = match prev_schedule_map.get(&b.team_id) {
        Some(prev) => prev.clone(),
        None => TeamScheduleData::default(),
    };

    return (prev_a, prev_b);
}

// Get the indexes of sort_functions in the wanted order.
fn get_sort_order(sort_type: &MatchGenType, rng: &mut ThreadRng ) -> [usize; 2] {
    match sort_type {
        MatchGenType::MatchCount => [1, 0],
        MatchGenType::Random => {
            let mut indexes: [usize; 2] = [0, 1];
            indexes.shuffle(rng);
            indexes
        }
        _ => {
            panic!("invalid MatchGenType in this context ({:?})", sort_type)
        }
    }
}

// Sort the schedule data according to various customisable options.
fn sort_with_options(schedule_data: &mut Vec<TeamScheduleData>, prev_schedule_map: &HashMap<TeamId, TeamScheduleData>, sort_type: &MatchGenType, rng: &mut ThreadRng, sort_functions: &[CmpFunc; 2]) {
    let indexes: [usize; 2] = get_sort_order(sort_type, rng);

    schedule_data.sort_by(|a: &TeamScheduleData, b: &TeamScheduleData| {
        let (prev_a, prev_b) = get_previous(a, b, prev_schedule_map);

        return sort_functions[indexes[0]](a, b, &prev_a, &prev_b)
            .then(sort_functions[indexes[1]](a, b, &prev_a, &prev_b));
    });
}

// Prioritise teams that need any game.
pub fn sort_default(sort_type: &MatchGenType, schedule_data: &mut Vec<TeamScheduleData>, prev_schedule_map: &HashMap<TeamId, TeamScheduleData>, rng: &mut ThreadRng) {
    let sort_functions: [CmpFunc; 2] = [compare_home_away_abs, compare_match_count];
    sort_with_options(schedule_data, prev_schedule_map, sort_type, rng, &sort_functions);
}

// Prioritise teams that need a home game.
pub fn sort_home(sort_type: &MatchGenType, schedule_data: &mut Vec<TeamScheduleData>, prev_schedule_map: &HashMap<TeamId, TeamScheduleData>, rng: &mut ThreadRng) {
    let sort_functions: [CmpFunc; 2] = [compare_home_away, compare_match_count];
    sort_with_options(schedule_data, prev_schedule_map, sort_type, rng, &sort_functions);
}

// Prioritise teams that need a home game.
pub fn sort_away(sort_type: &MatchGenType, schedule_data: &mut Vec<TeamScheduleData>, prev_schedule_map: &HashMap<TeamId, TeamScheduleData>, rng: &mut ThreadRng) {
    let sort_functions: [CmpFunc; 2] = [compare_away_home, compare_match_count];
    sort_with_options(schedule_data, prev_schedule_map, sort_type, rng, &sort_functions);
}