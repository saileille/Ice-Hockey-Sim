use crate::types::AttributeValue;


// Attribute data.
#[derive(Hash, PartialEq, Eq)]
#[derive(Debug)]
#[derive(Default, Clone)]
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
pub struct Attribute {
    id: AttributeId,
    peak: u8,
}

impl Attribute {
    pub fn build(id: AttributeId, peak: u8) -> Self {
        Self {
            id: id,
            peak: peak,
        }
    }
}

#[derive(Debug)]
#[derive(Default, Clone)]
pub struct PersonAttribute {
    id: AttributeId,
    value: AttributeValue,
}

impl PersonAttribute {
    // The attribute value limits inside the code.
    const MIN: AttributeValue = 19;
    const MAX: AttributeValue = AttributeValue::MAX;

    // Multiply by this amount when doing logarithmic stuff.
    // Currently 277 / 16. ((u8::MAX + 1 + MIN) / AttributeValue::BITS)
    const DISPLAY_MULTIPLIER: f64 = 17.1875;

    // Subtract by this amount when doing logarithmic stuff.
    // log2(MIN) * DISPLAY_MULTIPLIER
    const DISPLAY_SUBTRACTOR: f64 = 73.01125413731162;

    pub fn build(id: AttributeId, value: AttributeValue) -> Self {
        let mut attribute = Self::default();
        attribute.id = id;
        attribute.set(value);
        return attribute;
    }

    // Get a display value of the attribute.
    pub fn get_display(&self) -> u8 {
        // Now between 0 and 16.
        let mut log = (self.value as f64).log2();

        // Let's give some additional range.
        log *= Self::DISPLAY_MULTIPLIER;

        // Decrease the value so the minimum is 0.
        let attribute = (log - Self::DISPLAY_SUBTRACTOR) as u8;

        // Now between 0 and 201.
        return attribute;
    }

    pub fn get(&self) -> AttributeValue {
        return self.value;
    }

    // Set the attribute.
    fn set(&mut self, value: AttributeValue) {
        self.value = value.clamp(Self::MIN, Self::MAX);
    }
}