use time::Date;

use crate::logic::{game::event::PlayersOnIce, team::lineup::LineUp, types::{GameEventId, GameId, GameSeconds, PersonId, SeasonId, TeamId}};

struct Game {
    id: GameId,
    date: Date,
    clock: GameSeconds,
    home_id: TeamId,
    away_id: TeamId,
    season_id: SeasonId,
}

struct TeamGame {
    game_id: GameId,
    team_id: TeamId,
    lineup: LineUp,
}

struct GameEvent {
    id: GameEventId,
    game_id: GameId,
    target_team_id: TeamId,
    opponent_team_id: TeamId,
    time: GameSeconds,
    target_players: PlayersOnIce,
    opponent_players: PlayersOnIce,
}

struct ShotEvent {
    event_id: GameEventId,
    shooter_id: PersonId,
    assister_1_id: PersonId,
    assister_2_id: PersonId,
    is_goal: bool,
}