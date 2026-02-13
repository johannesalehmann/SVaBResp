use prism_model::{Expression, VariableManager, VariableReference};

pub struct ValuationMap {
    entries: Vec<ValuationMapEntry>,
}

impl ValuationMap {
    pub fn new<S: Clone>(variables: &VariableManager<Expression<VariableReference, S>, S>) -> Self {
        let mut entries = Vec::new();

        let mut variables_count = 0;
        let mut consts_cont = 0;
        for var in &variables.variables {
            if var.is_constant {
                entries.push(ValuationMapEntry::Const(consts_cont));
                consts_cont += 1;
            } else {
                entries.push(ValuationMapEntry::Var(variables_count));
                variables_count += 1;
            }
        }
        Self { entries }
    }

    pub fn map_to_variable(&self, index: usize) -> Option<usize> {
        match self.entries[index] {
            ValuationMapEntry::Const(_) => None,
            ValuationMapEntry::Var(i) => Some(i),
        }
    }

    #[allow(unused)]
    pub fn map_to_constant(&self, index: usize) -> Option<usize> {
        match self.entries[index] {
            ValuationMapEntry::Const(i) => Some(i),
            ValuationMapEntry::Var(_) => None,
        }
    }
}

impl std::ops::Index<usize> for ValuationMap {
    type Output = ValuationMapEntry;

    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

pub enum ValuationMapEntry {
    Const(usize),
    Var(usize),
}
