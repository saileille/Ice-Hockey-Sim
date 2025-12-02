// Round robin specific seasonal stuff.
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[derive(Default, Clone)]
pub struct RoundRobin {

}

impl RoundRobin {
    pub fn build() -> Self {
        Self::default()
    }
}