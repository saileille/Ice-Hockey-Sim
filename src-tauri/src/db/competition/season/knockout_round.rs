use sqlx::{Decode, Encode, Sqlite, encode::IsNull, error::BoxDynError, sqlite::{SqliteArgumentValue, SqliteTypeInfo, SqliteValueRef}};

use crate::logic::{competition::season::knockout_round::{KnockoutPair, KnockoutRound}, types::{Db, SeasonId}};

impl sqlx::Type<Sqlite> for KnockoutRound {
    fn type_info() -> SqliteTypeInfo {
        <String as sqlx::Type<Sqlite>>::type_info()
    }
}

impl<'q> Encode<'q, Sqlite> for KnockoutRound {
    fn encode(self, buf: &mut Vec<SqliteArgumentValue<'q>>) -> Result<IsNull, BoxDynError> {
        Encode::<Sqlite>::encode(serde_json::to_string(&self).unwrap(), buf)
    }

    fn encode_by_ref(&self, buf: &mut Vec<SqliteArgumentValue<'q>>) -> Result<IsNull, BoxDynError> {
        Encode::<Sqlite>::encode(serde_json::to_string(self).unwrap(), buf)
    }
}

impl<'r> Decode<'r, Sqlite> for KnockoutRound {
    fn decode(value: SqliteValueRef<'r>) -> Result<Self, BoxDynError> {
        let json = <serde_json::Value as Decode<Sqlite>>::decode(value)?;
        Ok(serde_json::from_value(json)?)
    }
}

impl KnockoutRound {
    pub async fn save(&self, db: &Db, season_id: SeasonId) {
        sqlx::query(
            "UPDATE Season SET knockout_round = $1 WHERE id = $2"
        ).bind(self)
        .bind(season_id)
        .execute(db).await.unwrap();
    }
}

impl KnockoutPair {
    // Remove any upcoming games from these two teams.
    // Consider deleting the indexes on home_id and away_id once a better method has been developed?
    pub async fn clean_up_games(&self, db: &Db) {
        sqlx::query(
            "DELETE FROM Game
            WHERE (home_id IN ($1, $2) OR away_id IN ($1, $2))
            AND unixepoch(date) > (
                SELECT unixepoch(value_data) FROM KeyValue
                WHERE key_name = 'today'
            )"
        ).bind(self.home.team_id)
        .bind(self.away.team_id)
        .execute(db).await.unwrap();
    }
}