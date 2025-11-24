pub mod cache;

use serde::{Deserialize, Serialize};
use sqlx::{Decode, Encode, Sqlite, encode::IsNull, error::BoxDynError, sqlite::{SqliteArgumentValue, SqliteTypeInfo, SqliteValueRef}};

use crate::{
    person::player::{
        Player,
        position::PositionId
    }, types::{Db, PersonId, TeamId}
};

// A line-up of players used in a match.
#[derive(Debug, Serialize, Deserialize)]
#[derive(Default, Clone)]
pub struct LineUp {
    gk_ids: [PersonId; 2],
    pub defence_pairs: [DefencePair; 4],
    pub forward_lines: [ForwardLine; 4],
}

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

    // Make sure the lineup is filled.
    pub fn is_full(&self) -> bool {
        if self.gk_ids.contains(&0) { return false; }

        for pair in self.defence_pairs.iter() {
            if !pair.is_full() { return false; }
        }

        for line in self.forward_lines.iter() {
            if !line.is_full() { return false; }
        }

        return true;
    }
}

impl LineUp {
    // Clear the lineup.
    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

impl LineUp {
    // Add players from a roster to the lineup.
    pub fn auto_add(&mut self, players: Vec<Player>) {
        for player in players {
            self.auto_add_player(player);
        }
    }

    // Add a player to the lineup.
    fn auto_add_player(&mut self, player: Player) {
        match player.position_id {
            PositionId::Goalkeeper => self.auto_add_gk(player),
            PositionId::LeftDefender => self.auto_add_ld(player),
            PositionId::RightDefender => self.auto_add_rd(player),
            PositionId::LeftWinger => self.auto_add_lw(player),
            PositionId::Centre => self.auto_add_c(player),
            PositionId::RightWinger => self.auto_add_rw(player),
            _ => return
        }
    }

    // Add a goalkeeper to the lineup.
    fn auto_add_gk(&mut self, player: Player) {
        for id in self.gk_ids.iter_mut() {
            if *id == 0 {
                *id = player.person.id;
                return;
            }
        }
    }

    // Add a left defender to the lineup.
    fn auto_add_ld(&mut self, player: Player) {
        for pair in self.defence_pairs.iter_mut() {
            if pair.ld_id == 0 {
                pair.ld_id = player.person.id;
                return;
            }
        }
    }

    // Add a left defender to the lineup.
    fn auto_add_rd(&mut self, player: Player) {
        for pair in self.defence_pairs.iter_mut() {
            if pair.rd_id == 0 {
                pair.rd_id = player.person.id;
                return;
            }
        }
    }

    // Add a left winger to the lineup.
    fn auto_add_lw(&mut self, player: Player) {
        for line in self.forward_lines.iter_mut() {
            if line.lw_id == 0 {
                line.lw_id = player.person.id;
                return;
            }
        }
    }

    // Add a centre to the lineup.
    fn auto_add_c(&mut self, player: Player) {
        for line in self.forward_lines.iter_mut() {
            if line.c_id == 0 {
                line.c_id = player.person.id;
                return;
            }
        }
    }

    // Add a right winger to the lineup.
    fn auto_add_rw(&mut self, player: Player) {
        for line in self.forward_lines.iter_mut() {
            if line.rw_id == 0 {
                line.rw_id = player.person.id;
                return;
            }
        }
    }
}

// A pair of defenders used in a line-up.
#[derive(Debug, Serialize, Deserialize)]
#[derive(Default, Clone)]
pub struct DefencePair {
    pub ld_id: PersonId,
    pub rd_id: PersonId,
}

impl DefencePair {  // Basics.
    // Get a clone of the left defender.
    async fn left_defender(&self, db: &Db) -> Option<Player> {
        Player::fetch_from_db(db, self.ld_id).await
    }

    // Get a clone of the right defender.
    async fn right_defender(&self, db: &Db) -> Option<Player> {
        Player::fetch_from_db(db, self.rd_id).await
    }

    // Make sure the defence pair is full.
    fn is_full(&self) -> bool {
        self.ld_id != 0 &&
        self.rd_id != 0
    }
}

// A line of forwards used in a line-up.
#[derive(Debug, Serialize, Deserialize)]
#[derive(Default, Clone)]
pub struct ForwardLine {
    pub lw_id: PersonId,
    pub c_id: PersonId,
    pub rw_id: PersonId,
}

impl ForwardLine {  // Basics.
    // Get a clone of the left winger.
    async fn left_winger(&self, db: &Db) -> Option<Player> {
        Player::fetch_from_db(db, self.lw_id).await
    }

    // Get a clone of the centre forward.
    async fn centre(&self, db: &Db) -> Option<Player> {
        Player::fetch_from_db(db , self.c_id).await
    }

    // Get a clone of the right winger.
    async fn right_winger(&self, db: &Db) -> Option<Player> {
        Player::fetch_from_db(db, self.rw_id).await
    }

    // Make sure the forward line is full.
    fn is_full(&self) -> bool {
        self.lw_id != 0 &&
        self.c_id != 0 &&
        self.rw_id != 0
    }
}