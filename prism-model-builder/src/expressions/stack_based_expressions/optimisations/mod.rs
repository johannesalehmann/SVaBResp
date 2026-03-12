mod const_optimisations;
pub use const_optimisations::get_const_optimisations;

mod operation_range;

use crate::expressions::stack_based_expressions::{Operation, StackBasedExpression};
pub use operation_range::{OperationRange, OperationRangeStack};
use prism_model::VariableReference;

#[derive(Debug)]
pub struct OperationView<'a, 'b, 'c, V: Clone> {
    operations: &'a mut Vec<Operation<V>>,
    index: &'b mut usize,
    stack_ranges: &'c mut OperationRangeStack,
}

impl<'a, 'b, 'c, V: Clone> OperationView<'a, 'b, 'c, V> {
    pub fn current_operation(&self) -> &Operation<V> {
        &self.operations[*self.index]
    }

    pub fn operand_range(&self, offset: usize) -> &OperationRange {
        &self.stack_ranges.elements[self.stack_ranges.elements.len() - (offset + 1)]
    }

    pub fn try_operand_range(&self, offset: usize) -> Option<&OperationRange> {
        if self.stack_ranges.elements.len() >= offset + 1 {
            Some(&self.stack_ranges.elements[self.stack_ranges.elements.len() - (offset + 1)])
        } else {
            None
        }
    }

    pub fn operation_range(&self, operands: usize) -> OperationRange {
        if operands == 0 {
            OperationRange::single(*self.index)
        } else {
            let mut range = self.operand_range(operands - 1).clone();
            for i in (0..operands - 1).rev() {
                range = OperationRange::from_sub_ranges(&range, self.operand_range(i));
            }
            OperationRange::from_sub_ranges(&range, &OperationRange::single(*self.index))
        }
    }

    pub fn single_operation_from_stack(&self, offset: usize) -> Option<&Operation<V>> {
        if self.stack_ranges.elements.len() <= offset {
            return None;
        }
        let range = self.operand_range(offset);
        if range.is_single() {
            Some(&self.operations[range.start])
        } else {
            None
        }
    }

    pub fn replace_operations<I: Iterator<Item = Operation<V>>>(
        &mut self,
        operands: usize,
        replace_by: I,
    ) {
        self.replace_operations_internal(operands, replace_by, false)
    }

    pub fn replace_operations_and_reprocess<I: Iterator<Item = Operation<V>>>(
        &mut self,
        operands: usize,
        replace_by: I,
    ) {
        self.replace_operations_internal(operands, replace_by, true)
    }
    fn replace_operations_internal<I: Iterator<Item = Operation<V>>>(
        &mut self,
        operands: usize,
        replace_by: I,
        reprocess: bool,
    ) {
        let replacement_range = self.operation_range(operands);

        let old_length = self.operations.len();
        self.operations.splice(&replacement_range, replace_by);
        let new_length = self.operations.len();

        for _ in 0..operands {
            self.stack_ranges.elements.pop();
        }
        let end = replacement_range.end + new_length - old_length;
        self.stack_ranges.elements.push(OperationRange {
            start: replacement_range.start,
            end,
        });
        *self.index = end;

        if reprocess {
            // TODO: This is not the most elegant approach: Ideally, we would only back up as far as
            // necessary. However, that requires careful handling of stack_ranges, which is
            // currently not done for simplicity. However, it shouldn't be impossible and would turn
            // optimisation from a worst-case quadratic algorithm into a linear one (assuming the
            // minimal backup is bounded by a constant, which it should be in most cases).
            *self.index = 0;
            self.stack_ranges.elements.clear();
        }
    }

    pub fn operations_by_range(&self, range: &OperationRange) -> Vec<Operation<V>> {
        self.operations[range.start..range.end]
            .iter()
            .cloned()
            .collect()
    }

    pub fn step(&mut self) {
        self.stack_ranges
            .execute_operation(&self.operations[*self.index], *self.index);
        *self.index += 1;
    }

    pub fn is_done(&self) -> bool {
        *self.index >= self.operations.len()
    }
}

pub trait Optimisation {
    fn apply(&self, view: &mut OperationView<VariableReference>) -> OptimisationResult;
}

#[derive(PartialEq)]
pub enum OptimisationResult {
    Applied,
    NotApplied,
}

pub fn apply_optimisations(
    expression: &mut StackBasedExpression<VariableReference>,
    optimisations: &Vec<Box<dyn Optimisation + '_>>,
) {
    let mut view = OperationView {
        operations: &mut expression.operations,
        index: &mut 0,
        stack_ranges: &mut OperationRangeStack::new(),
    };

    while !view.is_done() {
        let mut applied_some = false;
        for optimisation in optimisations {
            if optimisation.apply(&mut view) == OptimisationResult::Applied {
                applied_some = true;
                break;
            }
        }
        if !applied_some {
            view.step();
        }
    }
}

#[cfg(test)]
pub(crate) use tests::test_optimisation;

#[cfg(test)]
mod tests {
    macro_rules! test_optimisation {
        ($name:ident, $optimisation:ident, [$($operations:expr),*],[$($result:expr),*]) => {
            test_optimisation!($name, {$optimisation {}}, [$($operations),*], [$($result),*]);
            // paste:: paste! {
            //     #[test]
            //     fn $name() {
            //         let mut expression = crate::expressions::stack_based_expressions::StackBasedExpression::<prism_model::VariableReference>::new(
            //             vec![$($operations),*],
            //             crate::expressions::stack_based_expressions::ExpressionType::Int,
            //         );
            //         crate::expressions::stack_based_expressions::optimisations::apply_optimisations(&mut expression, &vec![Box::new($optimisation {})]);
            //         assert_eq!(expression.operations, vec![$($result),*]);
            //     }
            // }
        };
        ($name:ident, {$optimisation:expr}, [$($operations:expr),*],[$($result:expr),*]) => {
            paste:: paste! {
                #[test]
                fn $name() {
                    let mut expression = crate::expressions::stack_based_expressions::StackBasedExpression::<prism_model::VariableReference>::new(
                        vec![$($operations),*],
                        crate::expressions::stack_based_expressions::ExpressionType::Int,
                    );
                    crate::expressions::stack_based_expressions::optimisations::apply_optimisations(&mut expression, &vec![Box::new($optimisation)]);
                    assert_eq!(expression.operations, vec![$($result),*]);
                }
            }
        };
    }

    pub(crate) use test_optimisation;
}
