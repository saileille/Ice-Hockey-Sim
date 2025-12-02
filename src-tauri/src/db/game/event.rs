use std::num::NonZero;

use sqlx::{Decode, Encode, FromRow, Row, Sqlite, encode::IsNull, error::BoxDynError, sqlite::{SqliteArgumentValue, SqliteRow, SqliteTypeInfo, SqliteValueRef}};

use crate::logic::{game::event::{Event, PlayersOnIce, Shot}, types::Db};

impl sqlx::Type<Sqlite> for PlayersOnIce {
    fn type_info() -> SqliteTypeInfo {
        <String as sqlx::Type<Sqlite>>::type_info()
    }
}

impl<'q> Encode<'q, Sqlite> for PlayersOnIce {
    fn encode(self, buf: &mut Vec<SqliteArgumentValue<'q>>) -> Result<IsNull, BoxDynError> {
        Encode::<Sqlite>::encode(serde_json::to_string(&self).unwrap(), buf)
    }

    fn encode_by_ref(&self, buf: &mut Vec<SqliteArgumentValue<'q>>) -> Result<IsNull, BoxDynError> {
        Encode::<Sqlite>::encode(serde_json::to_string(self).unwrap(), buf)
    }
}

impl<'r> Decode<'r, Sqlite> for PlayersOnIce {
    fn decode(value: SqliteValueRef<'r>) -> Result<Self, BoxDynError> {
        let json = <serde_json::Value as Decode<Sqlite>>::decode(value)?;
        Ok(serde_json::from_value(json)?)
    }
}

impl Event {
    pub async fn save(&mut self, db: &Db) {
        self.id = sqlx::query_scalar(
            "INSERT INTO GameEvent
            (game_id, target_team_id, opponent_team_id, time, target_players, opponent_players)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id"
        ).bind(self.game_id)
        .bind(self.target_team_id)
        .bind(self.opponent_team_id)
        .bind(self.time)
        .bind(&self.target_players)
        .bind(&self.opponent_players)
        .fetch_one(db).await.unwrap();
    }
}

impl FromRow<'_, SqliteRow> for Shot {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        Ok(Self {
            event: Event::from_row(row)?,
            shooter_id: row.try_get("shooter_id")?,
            assister_1_id: match row.try_get("assister_1_id")? {
                Some(id) => id,
                None => 0,
            },
            assister_2_id: match row.try_get("assister_2_id")? {
                Some(id) => id,
                None => 0,
            },
            is_goal: row.try_get("is_goal")?,
        })
    }
}

impl Shot {
    pub const QUERY_TEAM_GOALS_IN_MATCH: &str = "
        (SELECT COUNT(*) FROM ShotEvent
        INNER JOIN GameEvent ON GameEvent.id = ShotEvent.event_id
        WHERE TeamGame.game_id = game_id
        AND target_team_id = TeamGame.team_id
        AND is_goal = TRUE)
    ";

    pub async fn save(&mut self, db: &Db) {
        self.event.save(db).await;
        sqlx::query(
            "INSERT INTO ShotEvent
            (event_id, shooter_id, assister_1_id, assister_2_id, is_goal)
            VALUES ($1, $2, $3, $4, $5)"
        ).bind(self.event.id)
        .bind(self.shooter_id)
        .bind(NonZero::new(self.assister_1_id))
        .bind(NonZero::new(self.assister_2_id))
        .bind(self.is_goal)
        .execute(db).await.unwrap();
    }
}