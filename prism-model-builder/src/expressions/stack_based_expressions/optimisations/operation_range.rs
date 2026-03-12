use super::super::Operation;
use std::collections::Bound;
use std::fmt::Formatter;
use std::ops::RangeBounds;

#[derive(Clone)]
pub struct OperationRange {
    pub start: usize,
    pub end: usize,
}

impl OperationRange {
    pub fn single(index: usize) -> Self {
        Self {
            start: index,
            end: index + 1,
        }
    }

    pub fn from_range(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn from_sub_ranges(first: &Self, second: &Self) -> Self {
        if first.end != second.start {
            panic!(
                "Cannot construct operation range from two non-consecutive operation ranges. First range: {:?}, second range: {:?}",
                first, second
            );
        }
        Self {
            start: first.start,
            end: second.end,
        }
    }

    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn is_single(&self) -> bool {
        self.start + 1 == self.end
    }
}

impl RangeBounds<usize> for &OperationRange {
    fn start_bound(&self) -> Bound<&usize> {
        Bound::Included(&self.start)
    }

    fn end_bound(&self) -> Bound<&usize> {
        Bound::Excluded(&self.end)
    }
}

impl std::fmt::Debug for OperationRange {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "OperationRange {{{}..{}}}", self.start, self.end)
    }
}

pub struct OperationRangeStack {
    pub elements: Vec<OperationRange>,
}

impl OperationRangeStack {
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }

    fn without_operands(&mut self, index: usize) {
        self.elements.push(OperationRange::single(index));
    }

    fn with_1_operand(&mut self, index: usize) {
        let operand = self.elements.pop().unwrap();
        self.elements.push(OperationRange::from_sub_ranges(
            &operand,
            &OperationRange::single(index),
        ));
    }

    fn with_2_operands(&mut self, index: usize) {
        let operand_b = self.elements.pop().unwrap();
        let operand_a = self.elements.pop().unwrap();
        let joint_operand = OperationRange::from_sub_ranges(&operand_a, &operand_b);
        self.elements.push(OperationRange::from_sub_ranges(
            &joint_operand,
            &OperationRange::single(index),
        ));
    }
    fn with_n_operands(&mut self, index: usize, n: usize) {
        if n == 0 {
            self.without_operands(index);
        } else {
            let mut operands = self.elements.pop().unwrap();
            for _ in 1..n {
                let operand = self.elements.pop().unwrap();
                operands = OperationRange::from_sub_ranges(&operand, &operands);
            }
            self.elements.push(OperationRange::from_sub_ranges(
                &operands,
                &OperationRange::single(index),
            ));
        }
    }

    pub fn execute_operation<V>(&mut self, operation: &Operation<V>, index: usize) {
        match operation {
            Operation::PushInt(_) => self.without_operands(index),
            Operation::PushFloat(_) => self.without_operands(index),
            Operation::PushBool(_) => self.without_operands(index),
            Operation::PushVarOrConstInt(_) => self.without_operands(index),
            Operation::PushVarOrConstFloat(_) => self.without_operands(index),
            Operation::PushVarOrConstBool(_) => self.without_operands(index),
            Operation::NegateInt => self.with_1_operand(index),
            Operation::NegateFloat => self.with_1_operand(index),
            Operation::IntToFloat => self.with_1_operand(index),
            Operation::MultiplyInt => self.with_2_operands(index),
            Operation::MultiplyFloat => self.with_2_operands(index),
            Operation::DivideInt => self.with_2_operands(index),
            Operation::DivideFloat => self.with_2_operands(index),
            Operation::AddInt => self.with_2_operands(index),
            Operation::AddFloat => self.with_2_operands(index),
            Operation::SubtractInt => self.with_2_operands(index),
            Operation::SubtractFloat => self.with_2_operands(index),
            Operation::LessThanInt => self.with_2_operands(index),
            Operation::LessThanFloat => self.with_2_operands(index),
            Operation::LessOrEqualInt => self.with_2_operands(index),
            Operation::LessOrEqualFloat => self.with_2_operands(index),
            Operation::GreaterThanInt => self.with_2_operands(index),
            Operation::GreaterThanFloat => self.with_2_operands(index),
            Operation::GreaterOrEqualInt => self.with_2_operands(index),
            Operation::GreaterOrEqualFloat => self.with_2_operands(index),
            Operation::EqualsInt => self.with_2_operands(index),
            Operation::EqualsFloat => self.with_2_operands(index),
            Operation::EqualsBool => self.with_2_operands(index),
            Operation::NotEqualsInt => self.with_2_operands(index),
            Operation::NotEqualsFloat => self.with_2_operands(index),
            Operation::NotEqualsBool => self.with_2_operands(index),
            Operation::NegateBool => self.with_1_operand(index),
            Operation::Conjunction => self.with_2_operands(index),
            Operation::Disjunction => self.with_2_operands(index),
            Operation::IfAndOnlyIf => self.with_2_operands(index),
            Operation::Implies => self.with_2_operands(index),
            Operation::TernaryInt => self.with_n_operands(index, 3),
            Operation::TernaryFloat => self.with_n_operands(index, 3),
            Operation::TernaryBool => self.with_n_operands(index, 3),
            Operation::MinInt(i) => self.with_n_operands(index, *i),
            Operation::MinFloat(i) => self.with_n_operands(index, *i),
            Operation::MaxInt(i) => self.with_n_operands(index, *i),
            Operation::MaxFloat(i) => self.with_n_operands(index, *i),
            Operation::Floor => self.with_1_operand(index),
            Operation::Ceil => self.with_1_operand(index),
            Operation::Round => self.with_1_operand(index),
            Operation::PowInt => self.with_2_operands(index),
            Operation::PowFloat => self.with_2_operands(index),
            Operation::Mod => self.with_2_operands(index),
            Operation::LogFloat => self.with_2_operands(index),
            Operation::SubExpression(_) => {
                panic!("Cannot apply optimisations to expressions that contain subexpressions")
            }
        }
    }
}

impl std::fmt::Debug for OperationRangeStack {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.elements.iter()).finish()
    }
}
