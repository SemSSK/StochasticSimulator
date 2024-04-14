use crate::molecule::Molecule;
use itertools::Itertools;

#[derive(Debug, Clone)]
pub struct ValueBoard {
    pub rows: Vec<Vec<usize>>,
    pub columns: Vec<String>,
}

impl ValueBoard {
    pub fn add_entry(&mut self, mols: &[Molecule], time: usize) {
        let mut map = (0..(self.columns.len() - 1))
            .map(|i| mols.iter().filter(|m| m.kind.uuid == i as u64).count())
            .collect::<Vec<_>>();
        map.push(time);
        self.rows.push(map);
    }
    pub fn convert_to_csv(&self) -> String {
        let mut csv = self.columns.join(", ");
        for v in &self.rows {
            csv.push('\n');
            csv.push_str(&v.into_iter().join(", "));
        }
        csv
    }
}
