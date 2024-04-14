use std::hash::Hash;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Element {
    pub uuid: u64,
    pub radius: f32,
    pub speed: f32,
}

impl Eq for Element {}

impl Hash for Element {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uuid.hash(state);
    }
}
