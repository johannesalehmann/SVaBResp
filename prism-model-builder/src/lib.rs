mod expressions;

use crate::expressions::{Evaluator, ValuationSource};
use prism_model::{
    Expression, Identifier, Model, Update, VariableManager, VariableRange, VariableReference,
};
use probabilistic_models::{
    Action, ActionCollection, AtomicPropositions, Builder, ContextBuilder, Distribution, MdpType,
    ModelTypes, ProbabilisticModel, State, Successor, Valuation, ValuationBuilder,
};
use std::collections::HashMap;
use std::marker::PhantomData;

use probabilistic_models::DistributionBuilder;
pub fn build_model<S: Clone>(
    model: &Model<(), Identifier<S>, VariableReference, S>,
    atomic_propositions: &[Expression<VariableReference, S>],
) -> Result<ProbabilisticModel<MdpType>, ModelBuildingError> {
    ExplicitModelBuilder::<MdpType, DefaultModelBuilderTypes>::run(model, atomic_propositions)
}

pub trait ModelBuilderTypes {
    type ExpressionEvaluator: Evaluator;
}

pub struct DefaultModelBuilderTypes {}
impl ModelBuilderTypes for DefaultModelBuilderTypes {
    type ExpressionEvaluator = expressions::TreeWalkingEvaluator;
}

pub struct StateInProgress<M: ModelTypes> {
    pub valuation: M::Valuation,
    pub actions: <M::ActionCollection as ActionCollection<M>>::Builder,
    pub atomic_propositions: M::AtomicPropositions,
}
pub struct ExplicitModelBuilder<M: ModelTypes, B: ModelBuilderTypes> {
    phantom_data: PhantomData<B>,
    states: Vec<StateInProgress<M>>,
    valuation_to_state: HashMap<M::Valuation, usize>,
    open_states: Vec<usize>,
    valuation_map: ValuationMap,
    consts: ConstValuations,
    variable_bounds: VariableBounds,
    context: <M::Valuation as Valuation>::ContextType,
}

impl<M: ModelTypes, B: ModelBuilderTypes> ExplicitModelBuilder<M, B> {
    pub fn run<S: Clone>(
        model: &Model<(), Identifier<S>, VariableReference, S>,
        atomic_propositions: &[Expression<VariableReference, S>],
    ) -> Result<ProbabilisticModel<M>, ModelBuildingError> {
        let (valuation_map, consts) =
            Self::prepare_valuation_map_and_consts(&model.variable_manager)?;
        let variable_bounds =
            Self::prepare_variable_bounds(&model.variable_manager, &consts, &valuation_map)?;
        let context = Self::prepare_valuation_context(model, &valuation_map, &variable_bounds);

        let synchronised_actions = SynchronisedActions::from_prism(model);

        let mut builder = Self {
            phantom_data: Default::default(),
            states: Vec::new(),
            valuation_to_state: HashMap::new(),
            open_states: Vec::new(),
            valuation_map,
            consts,
            variable_bounds,
            context,
        };

        builder.create_initial_states(model, atomic_propositions.len())?;

        while let Some(state) = builder.open_states.pop() {
            builder.process_state(state, &model, atomic_propositions, &synchronised_actions)?;
        }

        let mut result = ProbabilisticModel::new();
        for (i, state_in_progress) in builder.states.into_iter().enumerate() {
            let state = State {
                valuation: state_in_progress.valuation,
                actions: state_in_progress.actions.finish(),
                atomic_propositions: state_in_progress.atomic_propositions,
            };
            result.states.push(state);
        }

        Ok(result)
    }

    fn prepare_valuation_map_and_consts<S: Clone>(
        variables: &VariableManager<VariableReference, S>,
    ) -> Result<(ValuationMap, ConstValuations), ModelBuildingError> {
        let mut valuation_map = ValuationMap {
            entries: Vec::new(),
        };
        let mut const_valuations = ConstValuations {
            valuations: Vec::new(),
        };
        for var in &variables.variables {
            if var.is_constant {
                valuation_map
                    .entries
                    .push(ValuationMapEntry::Const(const_valuations.valuations.len()));

                let const_value_source: ConstRecursiveEvaluator<'_, S, B::ExpressionEvaluator> =
                    ConstRecursiveEvaluator {
                        variables,
                        phantom_data: Default::default(),
                    };
                let value = B::ExpressionEvaluator::create().evaluate_as_int(
                    var.initial_value
                        .as_ref()
                        .expect("Consts must have an initial value expression"),
                    &const_value_source,
                );
                println!("Const {} has value {}", var.name, value);
                const_valuations.valuations.push(ConstValuation::Int(value));
            } else {
                valuation_map.entries.push(ValuationMapEntry::Var(
                    valuation_map.entries.len() - const_valuations.valuations.len(),
                ))
            }
        }
        Ok((valuation_map, const_valuations))
    }

    fn prepare_variable_bounds<S: Clone>(
        variables: &VariableManager<VariableReference, S>,
        consts: &ConstValuations,
        valuation_map: &ValuationMap,
    ) -> Result<(VariableBounds), ModelBuildingError> {
        let const_value_source: ConstOnlyEvaluator<'_, '_, B::ExpressionEvaluator> =
            ConstOnlyEvaluator {
                valuation_map: &valuation_map,
                const_values: &consts,
                phantom_data: Default::default(),
            };

        let mut bounds = Vec::new();
        for (i, variable) in variables.variables.iter().enumerate() {
            if let ValuationMapEntry::Var(index) = valuation_map.entries[i] {
                bounds.push(match &variable.range {
                    VariableRange::BoundedInt { min, max, .. } => {
                        let min = B::ExpressionEvaluator::create()
                            .evaluate_as_int(min, &const_value_source);
                        let max = B::ExpressionEvaluator::create()
                            .evaluate_as_int(max, &const_value_source);
                        Some((min, max))
                    }
                    _ => None,
                });
            }
        }

        Ok(VariableBounds { bounds })
    }

    fn prepare_valuation_context<S: Clone>(
        model: &Model<(), Identifier<S>, VariableReference, S>,
        valuation_map: &ValuationMap,
        variable_bounds: &VariableBounds,
    ) -> <<M as ModelTypes>::Valuation as Valuation>::ContextType {
        let mut context_builder = M::Valuation::get_context_builder();
        for (i, var) in model.variable_manager.variables.iter().enumerate() {
            if let ValuationMapEntry::Var(var_index) = &valuation_map.entries[i] {
                match &var.range {
                    VariableRange::BoundedInt { min, max, .. } => {
                        if let Some((min, max)) = variable_bounds.bounds[*var_index] {
                            context_builder.register_bounded_int(min, max);
                        } else {
                            panic!("Variable bounds and valuation map are inconsistent");
                        }
                    }
                    VariableRange::UnboundedInt { .. } => context_builder.register_unbounded_int(),
                    VariableRange::Boolean { .. } => context_builder.register_bool(),
                    VariableRange::Float { .. } => context_builder.register_float(),
                }
            }
        }
        let context = context_builder.finish();
        context
    }

    fn get_or_add_state(
        &mut self,
        valuation: M::Valuation,
        atomic_proposition_len: usize,
    ) -> usize {
        let index = self.valuation_to_state.get(&valuation);
        // let index = self
        //     .states
        //     .iter()
        //     .enumerate()
        //     .filter(|(_, v)| v.valuation == valuation)
        //     .map(|(i, _)| i)
        //     .next();
        match index {
            Some(&index) => index,
            None => {
                let index = self.states.len();
                let action_builder: <M::ActionCollection as ActionCollection<M>>::Builder =
                    M::ActionCollection::get_builder();
                let atomic_propositions =
                    <(M::AtomicPropositions)>::get_empty(atomic_proposition_len);
                self.valuation_to_state.insert(valuation.clone(), index);
                self.states.push(StateInProgress {
                    valuation,
                    actions: action_builder,
                    atomic_propositions,
                });
                self.open_states.push(index);
                index
            }
        }
    }

    fn process_state<S: Clone>(
        &mut self,
        state: usize,
        model: &Model<(), Identifier<S>, VariableReference, S>,
        atomic_propositions: &[Expression<VariableReference, S>],
        synchronised_actions: &SynchronisedActions,
    ) -> Result<(), ModelBuildingError> {
        self.evaluate_atomic_propositions(state, atomic_propositions);

        for module_index in 0..model.modules.modules.len() {
            let module = &model.modules.modules[module_index];
            for command_index in 0..module.commands.len() {
                let command = &module.commands[command_index];
                if command.action.is_some() {
                    continue; // Synchronising actions are handled separately
                }
                let valuation = &self.states[state].valuation;
                let val_source = ConstsAndVars::new(&self.valuation_map, &self.consts, valuation);
                let guard =
                    B::ExpressionEvaluator::create().evaluate_as_bool(&command.guard, &val_source);
                if guard {
                    let mut distribution = <M::Distribution as Distribution>::get_builder();

                    for update_index in 0..command.updates.len() {
                        let valuation = &self.states[state].valuation;
                        let val_source =
                            ConstsAndVars::new(&self.valuation_map, &self.consts, valuation);
                        let update = &command.updates[update_index];
                        let probability = B::ExpressionEvaluator::create()
                            .evaluate_as_float(&update.probability, &val_source);
                        let new_valuation = self.apply_assignments(
                            &model.variable_manager,
                            valuation,
                            &val_source,
                            &[&update],
                        );

                        let index = self.get_or_add_state(new_valuation, atomic_propositions.len());
                        distribution.add_successor(Successor { probability, index })
                    }

                    self.states[state].actions.add_action(Action {
                        successors: distribution.finish(),
                    });
                }
            }
        }

        for synchronised_action in &synchronised_actions.actions {
            let valuation = &self.states[state].valuation;
            let val_source = ConstsAndVars::new(&self.valuation_map, &self.consts, valuation);

            let mut satisfied_guards_indicies = Vec::new();
            let mut all_satisfied = true;
            for action_module in &synchronised_action.participating_modules {
                let module = &model.modules.modules[action_module.module_index];
                let mut module_info = Vec::new();
                for &command_index in &action_module.command_indices {
                    let command = &module.commands[command_index];
                    let guard = B::ExpressionEvaluator::create()
                        .evaluate_as_bool(&command.guard, &val_source);
                    if guard {
                        module_info.push(command_index);
                    }
                }
                if module_info.is_empty() {
                    all_satisfied = false;
                }
                satisfied_guards_indicies.push(module_info);
            }

            let n = satisfied_guards_indicies.len();

            if n == 0 {
                panic!("Synchronised actions with zero associated modules are not yet supported (but they should be impossible to create anyways");
            }

            if all_satisfied {
                let modules = &synchronised_action.participating_modules;
                let mut indices = vec![0; n];
                while indices[0] < satisfied_guards_indicies[0].len() {
                    let mut command_indices = Vec::with_capacity(n);
                    for i in 0..n {
                        command_indices.push(satisfied_guards_indicies[i][indices[i]]);
                    }

                    let mut update_indices = vec![0; n];

                    let mut distribution = <M::Distribution as Distribution>::get_builder();

                    while update_indices[0]
                        < model.modules.modules[modules[0].module_index].commands
                            [command_indices[0]]
                            .updates
                            .len()
                            .max(1)
                    // max(1) is required because a synchronising action may have an empty update ("true")
                    {
                        let valuation = &self.states[state].valuation;
                        let val_source =
                            ConstsAndVars::new(&self.valuation_map, &self.consts, valuation);
                        let mut updates = Vec::new();
                        for i in 0..n {
                            let command = &model.modules.modules[modules[i].module_index].commands
                                [command_indices[i]];
                            if command.updates.len() > 0 {
                                updates.push(&command.updates[update_indices[i]]);
                            }
                        }
                        let new_valuation = self.apply_assignments(
                            &model.variable_manager,
                            valuation,
                            &val_source,
                            &updates[..],
                        );

                        let mut probability = 1.0;

                        for i in 0..n {
                            let command = &model.modules.modules[modules[i].module_index].commands
                                [command_indices[i]];
                            if command.updates.len() > 0 {
                                let ith_expression =
                                    &command.updates[update_indices[i]].probability;
                                let ith_probability = B::ExpressionEvaluator::create()
                                    .evaluate_as_float(ith_expression, &val_source);
                                probability *= ith_probability;
                            }
                        }

                        let index = self.get_or_add_state(new_valuation, atomic_propositions.len());
                        distribution.add_successor(Successor { probability, index });

                        for i in (0..n).rev() {
                            if update_indices[i] + 1
                                < model.modules.modules[modules[i].module_index].commands
                                    [command_indices[i]]
                                    .updates
                                    .len()
                            {
                                update_indices[i] += 1;
                                for j in i + 1..n {
                                    update_indices[j] = 0;
                                }
                                break;
                            } else {
                                if i == 0 {
                                    update_indices[0] += 1;
                                }
                            }
                        }
                    }

                    self.states[state].actions.add_action(Action {
                        successors: distribution.finish(),
                    });

                    for i in (0..n).rev() {
                        if indices[i] < satisfied_guards_indicies[i].len() {
                            indices[i] += 1;
                            for j in i + 1..n {
                                indices[j] = 0;
                            }
                        } else {
                            if i == 0 {
                                indices[0] += 1;
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn evaluate_atomic_propositions<S: Clone>(
        &mut self,
        state_index: usize,
        atomic_propositions: &[Expression<VariableReference, S>],
    ) {
        let state = &mut self.states[state_index];
        let val_source = ConstsAndVars::new(&self.valuation_map, &self.consts, &state.valuation);
        for (i, atomic_proposition) in atomic_propositions.iter().enumerate() {
            let is_true =
                B::ExpressionEvaluator::create().evaluate_as_bool(&atomic_proposition, &val_source);
            state.atomic_propositions.set_value(i, is_true);
        }
    }

    fn apply_assignments<S: Clone>(
        &self,
        variable_manager: &VariableManager<VariableReference, S>,
        valuation: &<M as ModelTypes>::Valuation,
        val_source: &ConstsAndVars<<M as ModelTypes>::Valuation>,
        updates: &[&Update<VariableReference, S>],
    ) -> <M as ModelTypes>::Valuation {
        let mut new_valuation = valuation.clone();
        for update in updates {
            for assignment in &update.assignments {
                let target = variable_manager.get(&assignment.target).unwrap();
                let target_index = match self.valuation_map.entries[assignment.target.index] {
                    ValuationMapEntry::Const(_) => panic!("Cannot assign to constant"),
                    ValuationMapEntry::Var(index) => index,
                };
                match target.range {
                    VariableRange::BoundedInt { .. } => {
                        let value = B::ExpressionEvaluator::create()
                            .evaluate_as_int(&assignment.value, &val_source);
                        let (min, max) = self.variable_bounds.bounds[target_index].unwrap();
                        if value < min || value > max {
                            panic!(
                                "Value exceeds variable bounds, bounds are ({}, {}), value is {}",
                                min, max, value
                            );
                        } else {
                            new_valuation.set_bounded_int(target_index, value);
                        }
                    }
                    VariableRange::UnboundedInt { .. } => {
                        let value = B::ExpressionEvaluator::create()
                            .evaluate_as_int(&assignment.value, &val_source);
                        new_valuation.set_unbounded_int(target_index, value);
                    }
                    VariableRange::Boolean { .. } => {
                        let value = B::ExpressionEvaluator::create()
                            .evaluate_as_bool(&assignment.value, &val_source);
                        new_valuation.set_bool(target_index, value);
                    }
                    VariableRange::Float { .. } => {
                        let value = B::ExpressionEvaluator::create()
                            .evaluate_as_float(&assignment.value, &val_source);
                        new_valuation.set_float(target_index, value);
                    }
                }
            }
        }
        new_valuation
    }

    fn create_initial_states<S: Clone>(
        &mut self,
        model: &Model<(), Identifier<S>, VariableReference, S>,
        atomic_proposition_len: usize,
    ) -> Result<(), ModelBuildingError> {
        if model.init_constraint.is_some() {
            panic!("Init constraints are not yet supported by the model builder");
        }
        let const_value_source: ConstOnlyEvaluator<'_, '_, B::ExpressionEvaluator> =
            ConstOnlyEvaluator {
                valuation_map: &self.valuation_map,
                const_values: &self.consts,
                phantom_data: Default::default(),
            };

        let mut valuation_builder = M::Valuation::get_builder(&self.context);

        for (i, variable) in model.variable_manager.variables.iter().enumerate() {
            if let ValuationMapEntry::Var(index) = &self.valuation_map.entries[i] {
                match variable.range {
                    VariableRange::BoundedInt { .. } => match &variable.initial_value {
                        None => {
                            if let Some((min, _)) = self.variable_bounds.bounds[*index] {
                                valuation_builder.add_bounded_int(min);
                            } else {
                                panic!("Variable bounds list is inconsistent");
                            }
                        }
                        Some(initial) => {
                            let value = B::ExpressionEvaluator::create()
                                .evaluate_as_int(initial, &const_value_source);
                            valuation_builder.add_bounded_int(value);
                        }
                    },
                    VariableRange::UnboundedInt { .. } => match &variable.initial_value {
                        None => panic!("Unbounded int must have init expression"),
                        Some(initial) => {
                            let value = B::ExpressionEvaluator::create()
                                .evaluate_as_int(initial, &const_value_source);
                            valuation_builder.add_int(value);
                        }
                    },
                    VariableRange::Boolean { .. } => match &variable.initial_value {
                        None => {
                            valuation_builder.add_bool(false);
                        }
                        Some(initial) => {
                            let value = B::ExpressionEvaluator::create()
                                .evaluate_as_bool(initial, &const_value_source);
                            valuation_builder.add_bool(value);
                        }
                    },
                    VariableRange::Float { .. } => match &variable.initial_value {
                        None => {
                            panic!("Floats must have init expressions (I'm not sure whether this is PRISM-spec-compliant)")
                        }
                        Some(initial) => {
                            let value = B::ExpressionEvaluator::create()
                                .evaluate_as_float(initial, &const_value_source);
                            valuation_builder.add_float(value);
                        }
                    },
                }
            }
        }

        let valuation = valuation_builder.finish();

        let index = self.get_or_add_state(valuation, atomic_proposition_len);
        println!("Initial state has index {}", index);

        Ok(())
    }

    fn print_valuation<S: Clone>(
        valuation: &M::Valuation,
        valuation_map: &ValuationMap,
        model: &Model<(), Identifier<S>, VariableReference, S>,
    ) {
        print!("(");
        let mut first = true;
        for (i, var) in model.variable_manager.variables.iter().enumerate() {
            if let ValuationMapEntry::Var(index) = valuation_map.entries[i] {
                if !first {
                    print!(", ");
                }
                first = false;
                print!("{} = ", var.name);
                match var.range {
                    VariableRange::BoundedInt { .. } => {
                        print!("{}", valuation.evaluate_bounded_int(index))
                    }
                    VariableRange::UnboundedInt { .. } => {
                        print!("{}", valuation.evaluate_bounded_int(index))
                    }
                    VariableRange::Boolean { .. } => {
                        print!("{}", valuation.evaluate_bool(index))
                    }
                    VariableRange::Float { .. } => {
                        print!("{}", valuation.evaluate_float(index))
                    }
                }
            }
        }
        print!(")");
    }
}

pub enum ModelBuildingError {}

struct ConstsAndVars<'a, 'b, 'c, V: Valuation> {
    map: &'a ValuationMap,
    consts: &'b ConstValuations,
    variables: &'c V,
}

impl<'a, 'b, 'c, V: Valuation> ConstsAndVars<'a, 'b, 'c, V> {
    pub fn new(map: &'a ValuationMap, consts: &'b ConstValuations, variables: &'c V) -> Self {
        Self {
            map,
            consts,
            variables,
        }
    }
}

impl<'a, 'b, 'c, V: Valuation> ValuationSource for &ConstsAndVars<'a, 'b, 'c, V> {
    fn get_int(&self, index: VariableReference) -> i64 {
        match self.map.entries[index.index] {
            ValuationMapEntry::Const(i) => self.consts.valuations[i].as_int(),
            ValuationMapEntry::Var(i) => self.variables.evaluate_bounded_int(i), // TODO: Also handle unbounded ints?
        }
    }

    fn get_bool(&self, index: VariableReference) -> bool {
        match self.map.entries[index.index] {
            ValuationMapEntry::Const(i) => self.consts.valuations[i].as_bool(),
            ValuationMapEntry::Var(i) => self.variables.evaluate_bool(i),
        }
    }

    fn get_float(&self, index: VariableReference) -> f64 {
        match self.map.entries[index.index] {
            ValuationMapEntry::Const(i) => self.consts.valuations[i].as_float(),
            ValuationMapEntry::Var(i) => self.variables.evaluate_float(i),
        }
    }
}

impl<'a, 'b, 'c, V: Valuation> ValuationSource for ConstsAndVars<'a, 'b, 'c, V> {
    fn get_int(&self, index: VariableReference) -> i64 {
        (&self).get_int(index)
    }

    fn get_bool(&self, index: VariableReference) -> bool {
        (&self).get_bool(index)
    }

    fn get_float(&self, index: VariableReference) -> f64 {
        (&self).get_float(index)
    }
}

struct VariableBounds {
    bounds: Vec<Option<(i64, i64)>>,
}

struct ValuationMap {
    entries: Vec<ValuationMapEntry>,
}

enum ValuationMapEntry {
    Const(usize),
    Var(usize),
}

struct ConstValuations {
    valuations: Vec<ConstValuation>,
}

enum ConstValuation {
    Int(i64),
    Bool(bool),
    Float(f64),
}

impl ConstValuation {
    pub fn as_int(&self) -> i64 {
        match self {
            ConstValuation::Int(i) => *i,
            _ => panic!("Cannot evaluate this value as integer"),
        }
    }

    pub fn as_bool(&self) -> bool {
        match self {
            ConstValuation::Bool(b) => *b,
            _ => panic!("Cannot evaluate this value as boolean"),
        }
    }

    pub fn as_float(&self) -> f64 {
        match self {
            ConstValuation::Float(f) => *f,
            _ => panic!("Cannot evaluate this value as float"),
        }
    }
}

struct ConstRecursiveEvaluator<'a, S: Clone, E: Evaluator> {
    variables: &'a VariableManager<VariableReference, S>,
    phantom_data: PhantomData<E>,
}

impl<'a, S: Clone, E: Evaluator> ValuationSource for ConstRecursiveEvaluator<'a, S, E> {
    fn get_int(&self, index: VariableReference) -> i64 {
        let var = self.variables.get(&index).unwrap();
        if !var.is_constant {
            panic!("Const depends on non-constant value");
        }
        let inner_eval = E::create();
        inner_eval.evaluate_as_int(
            &var.initial_value
                .as_ref()
                .expect("Constant without initial value"),
            self,
        )
    }

    fn get_bool(&self, index: VariableReference) -> bool {
        todo!()
    }

    fn get_float(&self, index: VariableReference) -> f64 {
        todo!()
    }
}

struct ConstOnlyEvaluator<'a, 'b, E: Evaluator> {
    valuation_map: &'a ValuationMap,
    const_values: &'b ConstValuations,
    phantom_data: PhantomData<E>,
}

impl<'a, 'b, E: Evaluator> ConstOnlyEvaluator<'a, 'b, E> {
    fn get(&self, index: VariableReference) -> &ConstValuation {
        match &self.valuation_map.entries[index.index] {
            ValuationMapEntry::Const(c) => &self.const_values.valuations[*c],
            ValuationMapEntry::Var(_) => {
                panic!("Cannot evaluate non-static value here");
            }
        }
    }
}
impl<'a, 'b, E: Evaluator> ValuationSource for ConstOnlyEvaluator<'a, 'b, E> {
    fn get_int(&self, index: VariableReference) -> i64 {
        self.get(index).as_int()
    }

    fn get_bool(&self, index: VariableReference) -> bool {
        self.get(index).as_bool()
    }

    fn get_float(&self, index: VariableReference) -> f64 {
        self.get(index).as_float()
    }
}

pub struct SynchronisedActions {
    actions: Vec<SynchronisedAction>,
}

pub struct SynchronisedAction {
    participating_modules: Vec<SynchronisedActionModule>,
}

pub struct SynchronisedActionModule {
    module_index: usize,
    command_indices: Vec<usize>,
}

impl SynchronisedActions {
    pub fn from_prism<S: Clone>(model: &Model<(), Identifier<S>, VariableReference, S>) -> Self {
        let mut actions: HashMap<String, SynchronisedAction> = HashMap::new();

        for (module_index, module) in model.modules.modules.iter().enumerate() {
            let mut module_actions: HashMap<String, SynchronisedActionModule> = HashMap::new();
            for (command_index, command) in module.commands.iter().enumerate() {
                if let Some(action) = &command.action {
                    if let Some(module_action) = module_actions.get_mut(&action.name) {
                        module_action.command_indices.push(command_index);
                    } else {
                        module_actions.insert(
                            action.name.clone(),
                            SynchronisedActionModule {
                                module_index,
                                command_indices: vec![command_index],
                            },
                        );
                    }
                }
            }
            for (action_name, module_action) in module_actions {
                if let Some(action) = actions.get_mut(&action_name) {
                    action.participating_modules.push(module_action);
                } else {
                    actions.insert(
                        action_name,
                        SynchronisedAction {
                            participating_modules: vec![module_action],
                        },
                    );
                }
            }
        }

        SynchronisedActions {
            actions: actions.into_iter().map(|(n, a)| a).collect(),
        }
    }
}
