use rand::{Rng, rngs::ThreadRng};

use crate::{db::ATTRIBUTES, logic::{time::years_to_days, types::{AttributeDisplayValue, AttributeValue}}};

// Attribute data.
#[derive(Hash, PartialEq, Eq)]
#[derive(Debug)]
#[derive(Default, Clone, Copy)]
pub enum AttributeId {
    Defending,
    Shooting,
    Passing,
    Faceoffs,

    // Placeholder for checks that do not yet have proper attribute(s) assigned to them.
    #[default]
    General,
}

// Contains data about the attribute itself:
// How quickly does it improve?
// When does it peak on average?
// Etc.
#[derive(Clone)]
pub struct Attribute {
    _id: AttributeId,

    // The age (in days) when the attribute starts to develop.
    start_change: u16,

    // The age (in days) when this attribute is usually at its peak.
    peak: u16
}

impl Attribute {
    pub fn build(id: AttributeId, start_change_year: u8, peak_year: u8) -> Self {
        Self {
            _id: id,
            start_change: years_to_days(start_change_year),
            peak: years_to_days(peak_year),
        }
    }

    fn fetch_from_db(id: AttributeId) -> Self {
        ATTRIBUTES.get(&id).unwrap().clone()
    }
}

#[derive(Debug)]
#[derive(Clone)]
pub struct PersonAttribute {
    id: AttributeId,
    pub value: AttributeValue,
}

impl Default for PersonAttribute {
    fn default() -> Self {
        Self {
            id: AttributeId::default(),
            value: Self::set_static(AttributeValue::default()),
        }
    }
}

impl PersonAttribute {
    // The attribute value limits inside the code.
    const MIN: AttributeValue = 19;
    const MAX: AttributeValue = AttributeValue::MAX;

    // Multiply by this amount when doing logarithmic stuff.
    const DISPLAY_MULTIPLIER: f64 = ((AttributeDisplayValue::MAX as AttributeValue + 1 + Self::MIN) / AttributeValue::BITS as AttributeValue) as f64;

    // Subtract by this amount when doing logarithmic stuff.
    // log2(MIN) * DISPLAY_MULTIPLIER
    const DISPLAY_SUBTRACTOR: f64 = 4.247927513443585 * Self::DISPLAY_MULTIPLIER;

    pub fn build(id: AttributeId, value: AttributeValue) -> Self {
        Self {
            id: id,
            value: Self::set_static(value),
        }
    }

    // Get a display value of the attribute.
    pub fn display(attribute: AttributeValue) -> AttributeDisplayValue {
        // Now between 0 and 16.
        let mut log = (attribute as f64).log2();

        // Let's give some additional range.
        log *= Self::DISPLAY_MULTIPLIER;

        // Decrease the value so the minimum is 0.
        let display_attribute = (log - Self::DISPLAY_SUBTRACTOR) as AttributeDisplayValue;

        // Now between 0 and 201.
        return display_attribute;
    }

    pub fn get(&self) -> AttributeValue {
        return self.value;
    }

    // Set the attribute.
    fn set(&mut self, value: AttributeValue) {
        self.value = Self::set_static(value);
    }

    fn set_static(value: AttributeValue) -> AttributeValue {
        return value.clamp(Self::MIN, Self::MAX);
    }

    // Change the attribute with the given value.
    fn change(&mut self, value: i32) {
        let changed_value = (self.value as i32) + value;
        self.set(changed_value as u16);
    }

    // The daily update check on the attribute.
    pub fn update(&mut self, rng: &mut ThreadRng, age_days: u16) {
        let attribute = Attribute::fetch_from_db(self.id);
        if age_days < attribute.start_change {
            return;
        }

        let regress_likelihood = (age_days as f64) / ((attribute.peak * 2) as f64);
        let attribute_regresses = rng.random_bool(regress_likelihood);

        if attribute_regresses {
            self.change(-1);
        }
        else {
            self.change(1);
        }
    }
}