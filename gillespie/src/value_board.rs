use itertools::Itertools;

#[derive(Debug, Clone)]
pub struct ValueBoard {
    pub rows: Vec<(Vec<i32>, f32)>,
    pub columns: Vec<String>,
}

impl ValueBoard {
    pub fn add_entry(&mut self, values: Vec<i32>, time: f32) {
        self.rows.push((values, time));
    }
    pub fn convert_to_csv(&self) -> String {
        let mut csv = self.columns.join(", ");
        for (v, t) in &self.rows {
            csv.push('\n');
            csv.push_str(&v.into_iter().join(", "));
            csv.push(',');
            csv.push_str(&t.to_string());
        }
        csv
    }
}
