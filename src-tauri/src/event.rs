// Events used for game logic, with likelihoods of something happening or not.
use rand::{
    rngs::ThreadRng,
    Rng
};

use crate::database;

#[derive(Eq, Hash, PartialEq)]
#[derive(Debug)]
pub enum Id {
    PuckPossessionChange,
    ShotAtGoal,
    Goal,
}

#[derive(Default, Clone)]
pub struct Type {
    min_boundary: f64,
    equilibrium: f64,
    max_boundary: f64,
}

impl Type {    // Basics.
    pub fn build(min_boundary: f64, equilibrium: f64, max_boundary: f64) -> Self {
        let mut event = Type::default();
        event.min_boundary = min_boundary;
        event.equilibrium = equilibrium;
        event.max_boundary = max_boundary;

        return event;
    }

    // Fetch the EventType from the database.
    pub fn fetch_from_db(id: &Id) -> Self {
        database::EVENT_TYPES.get(id).expect(&format!("no EventType with id {id:?}")).clone()
    }
}

impl Type {
    /*
    Function which forces a likelihood to adjust based on boundaries given to it.

    modifier: Float between 0.0 and 1.0.
    equilibrium: The resulting likelihood when modifier is 0.5.
    min_boundary: The resulting likelihood when modifier is 0.0.
    max_boundary: The resulting likelihood when modifier is 1.0.
    */
    fn calculate_likelihood(&self, modifier: f64) -> f64 {
        if modifier == 0.0 {return self.min_boundary}
        else if modifier == 0.5 {return self.equilibrium}
        else if modifier == 1.0 {return self.max_boundary}

        else if modifier > 0.5 {
            let percentage = modifier * 2.0 - 1.0;  // Percentage is between 0.0 and 1.0.
            let scale = self.max_boundary - self.equilibrium;    // The difference between 1.0 and 0.5 modifier.
            return self.equilibrium + scale * percentage;
        }

        else {
            let percentage = modifier * 2.0;   // Percentage is between 0.0 and 1.0.
            let scale = self.equilibrium - self.min_boundary;    // The difference between 0.5 and 0.0 modifier.
            return self.min_boundary + scale * percentage;
        }
    }

    // Get an outcome of the event that is either true or false.
    pub fn get_outcome(&mut self, modifier: f64) -> bool {
        let mut rng = rand::rng();
        return rng.random_bool(self.calculate_likelihood(modifier))
    }
}