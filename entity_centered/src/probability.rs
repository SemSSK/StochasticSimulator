#[derive(Debug)]
pub struct Probability(f64);

impl Probability {
    pub fn new(x: f64) -> Option<Self> {
        if x >= 0. && x <= 1. {
            Some(Probability(x))
        } else {
            None
        }
    }
    // Needs better error result with kcat and km
    // Should be on parser directly
    pub fn calc_probability(km: f32, kcat: f32) -> (Self, Self, Self) {
        let p3 = kcat / 10000.;
        let p2 = p3 / 10.;
        let p1 = if kcat >= 300. && km <= 80. {
            1.
        } else {
            (p2 + p3) / (0.448 * (1. + (p2 + p3).powi(2)) * km)
        };
        (
            Probability(p3 as f64),
            Probability(p2 as f64),
            Probability(p3 as f64),
        )
    }

    pub fn get(&self) -> f64 {
        self.0
    }
}
