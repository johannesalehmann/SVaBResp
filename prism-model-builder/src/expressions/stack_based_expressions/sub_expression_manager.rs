use crate::expressions::ValuationSource;
use crate::expressions::stack_based_expressions::{
    EvaluationStack, ExpressionType, StackBasedExpression,
};
use prism_model::VariableReference;

pub trait SubExpressionProvider {
    type EvaluationContext;
    fn create_context(&self) -> Self::EvaluationContext;
    fn reset_context(&self, context: &mut Self::EvaluationContext);

    fn get_sub_expression_type(
        &self,
        index: usize,
        context: &mut Self::EvaluationContext,
    ) -> super::ExpressionType;
    fn evaluate_as_int<VS: ValuationSource>(
        &self,
        index: usize,
        valuations: &VS,
        context: &mut Self::EvaluationContext,
    ) -> i64;
    fn evaluate_as_float<VS: ValuationSource>(
        &self,
        index: usize,
        valuations: &VS,
        context: &mut Self::EvaluationContext,
    ) -> f64;
    fn evaluate_as_bool<VS: ValuationSource>(
        &self,
        index: usize,
        valuations: &VS,
        context: &mut Self::EvaluationContext,
    ) -> bool;
}

pub struct EmptySubexpressionProvider {}

impl EmptySubexpressionProvider {
    pub fn new() -> Self {
        Self {}
    }
    pub fn panic<V>(&self) -> V {
        panic!("Cannot evaluate sub-expressions without providing a sub-expression manager");
    }
}

impl SubExpressionProvider for EmptySubexpressionProvider {
    type EvaluationContext = ();

    fn create_context(&self) -> Self::EvaluationContext {
        ()
    }

    fn reset_context(&self, context: &mut Self::EvaluationContext) {
        let _ = context;
    }

    fn get_sub_expression_type(
        &self,
        index: usize,
        context: &mut Self::EvaluationContext,
    ) -> ExpressionType {
        let _ = (index, context);
        self.panic()
    }

    fn evaluate_as_int<VS: ValuationSource>(
        &self,
        index: usize,
        valuations: &VS,
        context: &mut Self::EvaluationContext,
    ) -> i64 {
        let _ = (index, valuations, context);
        self.panic()
    }

    fn evaluate_as_float<VS: ValuationSource>(
        &self,
        index: usize,
        valuations: &VS,
        context: &mut Self::EvaluationContext,
    ) -> f64 {
        let _ = (index, valuations, context);
        self.panic()
    }

    fn evaluate_as_bool<VS: ValuationSource>(
        &self,
        index: usize,
        valuations: &VS,
        context: &mut Self::EvaluationContext,
    ) -> bool {
        let _ = (index, valuations, context);
        self.panic()
    }
}

pub struct ContextWithCache {
    base_context:
        <SubExpressionManager<VariableReference> as SubExpressionProvider>::EvaluationContext,
    cache: super::sub_expression_cache::SubExpressionCache,
}

pub struct SubExpressionManagerWithCache<V> {
    manager: SubExpressionManager<V>,
}

impl<V> SubExpressionManagerWithCache<V> {
    pub fn new(manager: SubExpressionManager<V>) -> Self {
        Self { manager }
    }
}

impl SubExpressionProvider for SubExpressionManagerWithCache<VariableReference> {
    type EvaluationContext = ContextWithCache;

    fn create_context(&self) -> Self::EvaluationContext {
        let cache = super::sub_expression_cache::SubExpressionCache::new(
            self.manager.sub_expressions.len(),
        );
        ContextWithCache {
            base_context: self.manager.create_context(),
            cache,
        }
    }

    fn reset_context(&self, context: &mut Self::EvaluationContext) {
        self.manager.reset_context(&mut context.base_context);
        context.cache.clear();
    }

    fn get_sub_expression_type(
        &self,
        index: usize,
        context: &mut Self::EvaluationContext,
    ) -> ExpressionType {
        self.manager
            .get_sub_expression_type(index, &mut context.base_context)
    }

    fn evaluate_as_int<VS: ValuationSource>(
        &self,
        index: usize,
        valuations: &VS,
        context: &mut Self::EvaluationContext,
    ) -> i64 {
        if let Some(val) = context.cache.get_int(index) {
            val
        } else {
            let mut stack = context.base_context.get_stack();
            let val = self.manager.sub_expressions[index]
                .evaluate_as_int_with_stack_and_sub_expressions(
                    valuations, self, &mut stack, context,
                );
            context.base_context.return_stack(stack);
            context.cache.store_int(index, val);
            val
        }
    }

    fn evaluate_as_float<VS: ValuationSource>(
        &self,
        index: usize,
        valuations: &VS,
        context: &mut Self::EvaluationContext,
    ) -> f64 {
        if let Some(val) = context.cache.get_float(index) {
            val
        } else {
            let mut stack = context.base_context.get_stack();
            let val = self.manager.sub_expressions[index]
                .evaluate_as_float_with_stack_and_sub_expressions(
                    valuations, self, &mut stack, context,
                );
            context.base_context.return_stack(stack);
            context.cache.store_float(index, val);
            val
        }
    }

    fn evaluate_as_bool<VS: ValuationSource>(
        &self,
        index: usize,
        valuations: &VS,
        context: &mut Self::EvaluationContext,
    ) -> bool {
        if let Some(val) = context.cache.get_bool(index) {
            val
        } else {
            let mut stack = context.base_context.get_stack();
            let val = self.manager.sub_expressions[index]
                .evaluate_as_bool_with_stack_and_sub_expressions(
                    valuations, self, &mut stack, context,
                );
            context.base_context.return_stack(stack);
            context.cache.store_bool(index, val);
            val
        }
    }
}

pub struct SubExpressionManagerEvaluationCache {
    stacks: Vec<EvaluationStack>,
}

impl SubExpressionManagerEvaluationCache {
    pub fn new() -> Self {
        Self { stacks: Vec::new() }
    }

    pub fn get_stack(&mut self) -> EvaluationStack {
        if let Some(mut stack) = self.stacks.pop() {
            stack.clear();
            stack
        } else {
            EvaluationStack::new()
        }
    }

    pub fn return_stack(&mut self, stack: EvaluationStack) {
        self.stacks.push(stack);
    }
}

pub struct SubExpressionManager<V> {
    sub_expressions: Vec<StackBasedExpression<V>>,
}

impl<V> SubExpressionManager<V> {
    pub fn new() -> Self {
        Self {
            sub_expressions: Vec::new(),
        }
    }

    pub fn add_sub_expression(
        &mut self,
        expression: StackBasedExpression<V>,
    ) -> StackBasedExpression<V> {
        let index = self.sub_expressions.len();
        let sub_expression =
            StackBasedExpression::with_sub_expression(index, expression.expression_type);
        self.sub_expressions.push(expression);
        sub_expression
    }
}

impl SubExpressionProvider for SubExpressionManager<VariableReference> {
    type EvaluationContext = SubExpressionManagerEvaluationCache;

    fn create_context(&self) -> Self::EvaluationContext {
        SubExpressionManagerEvaluationCache::new()
    }

    fn reset_context(&self, context: &mut Self::EvaluationContext) {
        let _ = context; // Context does not need to be reset, as it dynamically resets the individual stacks it provides
    }

    fn get_sub_expression_type(
        &self,
        index: usize,
        context: &mut Self::EvaluationContext,
    ) -> ExpressionType {
        let _ = context;
        self.sub_expressions[index].expression_type
    }

    fn evaluate_as_int<VS: ValuationSource>(
        &self,
        index: usize,
        valuations: &VS,
        context: &mut Self::EvaluationContext,
    ) -> i64 {
        let mut stack = context.get_stack();
        let int = self.sub_expressions[index]
            .evaluate_as_int_with_stack_and_sub_expressions(valuations, self, &mut stack, context);
        context.return_stack(stack);
        int
    }
    fn evaluate_as_float<VS: ValuationSource>(
        &self,
        index: usize,
        valuations: &VS,
        context: &mut Self::EvaluationContext,
    ) -> f64 {
        let mut stack = context.get_stack();
        let float = self.sub_expressions[index].evaluate_as_float_with_stack_and_sub_expressions(
            valuations, self, &mut stack, context,
        );
        context.return_stack(stack);
        float
    }
    fn evaluate_as_bool<VS: ValuationSource>(
        &self,
        index: usize,
        valuations: &VS,
        context: &mut Self::EvaluationContext,
    ) -> bool {
        let mut stack = context.get_stack();
        let boolean = self.sub_expressions[index]
            .evaluate_as_bool_with_stack_and_sub_expressions(valuations, self, &mut stack, context);
        context.return_stack(stack);
        boolean
    }
}
