use crate::types::AttributeValue;


// Attribute data.
#[derive(Hash, PartialEq, Eq)]
pub enum AttributeId {
    Defending,
    Shooting,
    Passing,
    Faceoffs,

    // Placeholder for checks that do not yet have proper attribute(s) assigned to them.
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

struct PersonAttribute {
    id: AttributeId,
    value: AttributeValue,
}