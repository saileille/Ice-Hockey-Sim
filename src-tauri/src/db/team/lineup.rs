use sqlx::{Decode, Encode, Sqlite, encode::IsNull, error::BoxDynError, sqlite::{SqliteArgumentValue, SqliteTypeInfo, SqliteValueRef}};

use crate::logic::{team::lineup::{DefencePair, ForwardLine, LineUp, cache::{DefencePairCache, ForwardLineCache, LineUpCache}}, types::{Db, TeamId}};

impl sqlx::Type<Sqlite> for LineUp {
    fn type_info() -> SqliteTypeInfo {
        <String as sqlx::Type<Sqlite>>::type_info()
    }
}

impl<'q> Encode<'q, Sqlite> for LineUp {
    fn encode(self, buf: &mut Vec<SqliteArgumentValue<'q>>) -> Result<IsNull, BoxDynError> {
        Encode::<Sqlite>::encode(serde_json::to_string(&self).unwrap(), buf)
    }

    fn encode_by_ref(&self, buf: &mut Vec<SqliteArgumentValue<'q>>) -> Result<IsNull, BoxDynError> {
        Encode::<Sqlite>::encode(serde_json::to_string(self).unwrap(), buf)
    }
}

impl<'r> Decode<'r, Sqlite> for LineUp {
    fn decode(value: SqliteValueRef<'r>) -> Result<Self, BoxDynError> {
        let json = <serde_json::Value as Decode<Sqlite>>::decode(value)?;
        Ok(serde_json::from_value(json)?)
    }
}

impl LineUp {
    // Save the lineup to the database.
    pub async fn save(&self, id: TeamId, db: &Db) {
        sqlx::query(
            "UPDATE Team SET lineup = $1
            WHERE id = $2"
        ).bind(self)
        .bind(id)
        .execute(db).await.unwrap();
    }
}

impl sqlx::Type<Sqlite> for LineUpCache {
    fn type_info() -> SqliteTypeInfo {
        <String as sqlx::Type<Sqlite>>::type_info()
    }
}

impl<'q> Encode<'q, Sqlite> for LineUpCache {
    fn encode(self, buf: &mut Vec<SqliteArgumentValue<'q>>) -> Result<IsNull, BoxDynError> {
        Encode::<Sqlite>::encode(serde_json::to_string(&self.to_db()).unwrap(), buf)
    }

    fn encode_by_ref(&self, buf: &mut Vec<SqliteArgumentValue<'q>>) -> Result<IsNull, BoxDynError> {
        Encode::<Sqlite>::encode(serde_json::to_string(&self.to_db()).unwrap(), buf)
    }
}

impl LineUpCache {
    fn to_db(&self) -> LineUp {
        let mut lineup = LineUp::default();

        for (i, gk) in self.goalkeepers.iter().enumerate() {
            lineup.gk_ids[i] = match gk {
                Some(player) => player.person.id,
                None => 0
            }
        }

        for (i, pair) in self.defence_pairs.iter().enumerate() {
            lineup.defence_pairs[i] = pair.to_db();
        }

        for (i, line) in self.forward_lines.iter().enumerate() {
            lineup.forward_lines[i] = line.to_db();
        }

        return lineup;
    }
}

impl DefencePairCache {
    fn to_db(&self) -> DefencePair {
        DefencePair {
            ld_id: match &self.ld {
                Some(player) => player.person.id,
                None => 0
            },
            rd_id: match &self.rd {
                Some(player) => player.person.id,
                None => 0
            }
        }
    }
}

impl ForwardLineCache {
    fn to_db(&self) -> ForwardLine {
        ForwardLine {
            lw_id: match &self.lw {
                Some(player) => player.person.id,
                None => 0
            },
            c_id: match &self.c {
                Some(player) => player.person.id,
                None => 0
            },
            rw_id: match &self.rw {
                Some(player) => player.person.id,
                None => 0
            }
        }
    }
}