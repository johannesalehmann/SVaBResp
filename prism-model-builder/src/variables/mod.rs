mod const_only_valuation_source;
pub use const_only_valuation_source::ConstOnlyValuationSource;

mod valuation_source;
pub use valuation_source::ConstAndVarValuationSource;

mod const_valuations;
mod valuation_map;
mod variable_details;

use const_valuations::*;
use valuation_map::*;

use crate::variables::variable_details::VariableDetails;
use crate::{ModelBuildingError, UserProvidedConstValue};
use prism_model::{Expression, Identifier, Model, VariableRange, VariableReference};
use probabilistic_models::{ContextBuilder, Valuation};
use std::collections::HashMap;

pub struct ModelVariableInfo<V: Valuation> {
    pub valuation_map: ValuationMap,
    const_valuations: ConstValuations,
    pub details: VariableDetails,
    pub valuation_context: V::ContextType,
}

impl<V: Valuation> ModelVariableInfo<V> {
    pub fn new<S: Clone>(
        model: &Model<(), Identifier<S>, Expression<VariableReference, S>, VariableReference, S>,
        user_provided_consts: &HashMap<String, UserProvidedConstValue>,
    ) -> Result<Self, ModelBuildingError> {
        let variables = &model.variable_manager;

        let valuation_map = ValuationMap::new(variables);
        let const_valuations = ConstValuations::new(variables, user_provided_consts);
        let details = VariableDetails::new(variables, &valuation_map, &const_valuations);
        let valuation_context = Self::prepare_valuation_context(model, &valuation_map, &details);

        Ok(Self {
            valuation_map,
            const_valuations,
            details,
            valuation_context,
        })
    }

    fn prepare_valuation_context<S: Clone>(
        model: &Model<(), Identifier<S>, Expression<VariableReference, S>, VariableReference, S>,
        valuation_map: &ValuationMap,
        details: &VariableDetails,
    ) -> V::ContextType {
        let mut context_builder = V::get_context_builder();
        for (i, var) in model.variable_manager.variables.iter().enumerate() {
            if let ValuationMapEntry::Var(var_index) = &valuation_map[i] {
                match &var.range {
                    VariableRange::BoundedInt { .. } => {
                        if let Some((min, max)) = details[*var_index].bounds {
                            context_builder.register_bounded_int(var.name.name.clone(), min, max);
                        } else {
                            panic!("Variable bounds and valuation map are inconsistent");
                        }
                    }
                    VariableRange::UnboundedInt { .. } => {
                        context_builder.register_unbounded_int(var.name.name.clone())
                    }
                    VariableRange::Boolean { .. } => {
                        context_builder.register_bool(var.name.name.clone())
                    }
                    VariableRange::Float { .. } => {
                        context_builder.register_float(var.name.name.clone())
                    }
                }
            }
        }
        let context = context_builder.finish();
        context
    }

    pub fn get_const_only_valuation_source(&self) -> ConstOnlyValuationSource<'_, '_> {
        ConstOnlyValuationSource::new(&self.valuation_map, &self.const_valuations)
    }

    pub fn get_valuation_source<'a, 'b, V2: Valuation>(
        &'a self,
        valuation: &'b V2,
    ) -> ConstAndVarValuationSource<'a, 'a, 'a, 'b, V2> {
        ConstAndVarValuationSource::new(
            &self.valuation_map,
            &self.const_valuations,
            &self.details,
            valuation,
        )
    }
}
