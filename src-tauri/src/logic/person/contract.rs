use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::FromRow;
use time::{Date, Duration};

use crate::logic::{competition::Competition, team::Team, time::{date_to_string, years_between}, types::{Db, PersonId, TeamId}};

#[derive(Copy, Clone, Debug)]
#[derive(Serialize, Deserialize)]
#[derive(sqlx::Type)]
pub enum ContractRole {
    Player,
    Manager,
}

// Contract a person has with a club.
#[derive(Clone, Debug)]
#[derive(FromRow)]
pub struct Contract {
    pub person_id: PersonId,
    pub team_id: TeamId,
    #[sqlx(rename = "begin_date")]
    pub start_date: Date,
    pub end_date: Date,
    pub role: ContractRole,
    pub is_signed: bool,
}

impl Contract {
    // Create a contract.
    pub async fn build_and_save(db: &Db, person_id: PersonId, team_id: TeamId, start_date: Date, end_date: Date, role: ContractRole, is_signed: bool) -> Self {
        let contract = Self {
            person_id,
            team_id,
            start_date,
            end_date,
            role,
            is_signed,
        };

        contract.save_new(db).await;
        return contract;
    }

    // Create a contract based on the team and how many years it should last.
    pub async fn build_from_years(db: &Db, today: Date, person_id: PersonId, team: &Team, years: i32, role: ContractRole, is_signed: bool) -> Self {
        let comp = Competition::fetch_from_db(db, team.primary_comp_id).await;
        let end_date = comp.season_window.end.previous_date_with_year_offset(today, years);

        return Self::build_and_save(db, person_id, team.id, today, end_date, role, is_signed).await;
    }

    // How many days there are left of the contract.
    fn _days_left(&self, today: Date) -> i64 {
        self._duration_left(today).whole_days()
    }

    // How many days have expired from the contract.
    pub fn days_expired(&self, today: Date) -> i64 {
        return self.duration_expired(today).whole_days()
    }

    // How many seasons there are left of the contract.
    // Note that 1 means less than a year left of the contract!
    fn seasons_left(&self, today: Date) -> i8 {
        return years_between(today, self.end_date) + 1;
    }

    // Get how much is left of the contract.
    fn _duration_left(&self, today: Date) -> Duration {
        return self.end_date - today;
    }

    // Get how much has expired of the contract.
    fn duration_expired(&self, today: Date) -> Duration {
        return today - self.start_date;
    }

    // Get relevant information for frontend.
    pub async fn package(&self, db: &Db, today: Date) -> serde_json::Value {
        json!({
            "start_date": date_to_string(self.start_date),
            "end_date": date_to_string(self.end_date),
            "seasons_left": self.seasons_left(today),
            "team": self.team(db).await.contract_package()
        })
    }
}