use crate::expressions::VariableType;
use crate::variables::const_valuations::{ConstValuation, ConstValuations};
use crate::variables::valuation_map::{ValuationMap, ValuationMapEntry};
use prism_model::VariableReference;

pub struct ConstOnlyValuationSource<'a, 'b> {
    valuation_map: &'a ValuationMap,
    const_values: &'b ConstValuations,
}

impl<'a, 'b> ConstOnlyValuationSource<'a, 'b> {
    pub fn new(valuation_map: &'a ValuationMap, const_values: &'b ConstValuations) -> Self {
        Self {
            valuation_map,
            const_values,
        }
    }

    fn get(&self, index: VariableReference) -> &ConstValuation {
        match &self.valuation_map[index.index] {
            ValuationMapEntry::Const(c) => &self.const_values[*c],
            ValuationMapEntry::Var(_) => {
                panic!("Cannot evaluate non-static value here");
            }
        }
    }
}
impl<'a, 'b> crate::ValuationSource for ConstOnlyValuationSource<'a, 'b> {
    fn get_int(&self, index: VariableReference) -> i64 {
        self.get(index).as_int()
    }

    fn get_bool(&self, index: VariableReference) -> bool {
        self.get(index).as_bool()
    }

    fn get_float(&self, index: VariableReference) -> f64 {
        self.get(index).as_float()
    }

    fn get_type(&self, index: VariableReference) -> VariableType {
        match self.get(index) {
            ConstValuation::Int(_) => VariableType::Int,
            ConstValuation::Bool(_) => VariableType::Bool,
            ConstValuation::Float(_) => VariableType::Float,
        }
    }
}
