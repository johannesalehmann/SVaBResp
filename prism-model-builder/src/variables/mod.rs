mod const_only_valuation_source;
pub use const_only_valuation_source::ConstOnlyValuationSource;

mod valuation_source;
pub use valuation_source::ConstAndVarValuationSource;

mod const_valuations;
pub use const_valuations::ConstValuation;

mod valuation_map;
mod variable_details;

use const_valuations::*;
use valuation_map::*;

use crate::variables::variable_details::VariableDetails;
use crate::{ExpressionContext, ModelBuildingError, UserProvidedConstValue};
use prism_model::{Identifier, Model, VariableRange, VariableReference};
use probabilistic_models::{ContextBuilder, Valuation};
use std::collections::HashMap;

pub struct ModelVariableInfo<V: Valuation> {
    pub valuation_map: ValuationMap,
    const_valuations: ConstValuations,
    pub details: VariableDetails,
    pub valuation_context: V::ContextType,
}

impl<V: Valuation> ModelVariableInfo<V> {
    /// Generates a valuation source for use in tests.
    ///
    /// The valuation source contains the following items.
    ///
    /// * `[0]`: Variable with index 0 of type `float` with name `float_var`
    /// * `[1]`: Variable with index 1 of type `int` with name `int_var` and bounds -10, 15
    /// * `[2]`: Const with index 0 of type `int` with value `-5`
    /// * `[3]`: Const with index 1 of type `bool` with value `true`
    /// * `[4]`: Variable with index 2 of type `bool` with name `bool_var`
    /// * `[5]`: Const with index 2 of type `float` with value `1.23`
    #[cfg(test)]
    pub fn with_mock_values() -> Self {
        let mut builder = V::get_context_builder();
        builder.register_float("float_var".to_string());
        builder.register_bounded_int("int_var".to_string(), -10, 15);
        builder.register_bool("bool_var".to_string());

        ModelVariableInfo {
            valuation_map: ValuationMap::with_mock_values(),
            const_valuations: ConstValuations::with_mock_values(),
            details: VariableDetails::with_mock_values(),
            valuation_context: builder.finish(),
        }
    }

    pub fn new<S: Clone, E, EC: ExpressionContext<E>>(
        model: &Model<(), Identifier<S>, E, VariableReference, S>,
        user_provided_consts: &HashMap<String, UserProvidedConstValue>,
        expression_context: &mut EC,
    ) -> Result<Self, ModelBuildingError> {
        let variables = &model.variable_manager;

        let valuation_map = ValuationMap::new(variables);
        let const_valuations =
            ConstValuations::new(variables, user_provided_consts, expression_context);
        let details = VariableDetails::new(
            variables,
            &valuation_map,
            &const_valuations,
            expression_context,
        );
        let valuation_context = Self::prepare_valuation_context(model, &valuation_map, &details);

        Ok(Self {
            valuation_map,
            const_valuations,
            details,
            valuation_context,
        })
    }

    fn prepare_valuation_context<S: Clone, E>(
        model: &Model<(), Identifier<S>, E, VariableReference, S>,
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

    pub fn value_of_const(&self, reference: VariableReference) -> Option<ConstValuation> {
        Some(self.const_valuations[self.valuation_map.map_to_constant(reference.index)?])
    }
}
