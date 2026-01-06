use crate::expressions::ValuationSource;
use prism_model::{Expression, VariableManager, VariableRange, VariableReference};

#[derive(PartialEq)]
enum ExpressionType {
    Int,
    Bool,
    Float,
}

pub struct StackBasedExpression<V> {
    operations: Vec<Operation<V>>,
}

impl StackBasedExpression<VariableReference> {
    pub fn from_expression<S: Clone>(
        expression: Expression<VariableReference, S>,
        variable_manager: VariableManager<VariableReference, S>,
    ) -> Self {
        let mut operations = Vec::new();

        Self { operations }
    }

    fn process_expression<S: Clone>(
        expression: &Expression<VariableReference, S>,
        operations: &mut Vec<Operation<VariableReference>>,
        variable_manager: &VariableManager<VariableReference, S>,
    ) -> ExpressionType {
        match expression {
            Expression::Int(i, _) => {
                operations.push(Operation::PushInt(*i));
                ExpressionType::Int
            }
            Expression::Float(f, _) => {
                operations.push(Operation::PushFloat(*f));
                ExpressionType::Float
            }
            Expression::Bool(b, _) => {
                operations.push(Operation::PushBool(*b));
                ExpressionType::Bool
            }
            Expression::VarOrConst(id, _) => match variable_manager.variables[id.index].range {
                VariableRange::BoundedInt { .. } | VariableRange::UnboundedInt { .. } => {
                    operations.push(Operation::PushVarOrConstInt(*id));
                    ExpressionType::Int
                }
                VariableRange::Float { .. } => {
                    operations.push(Operation::PushVarOrConstFloat(*id));
                    ExpressionType::Float
                }
                VariableRange::Boolean { .. } => {
                    operations.push(Operation::PushVarOrConstBool(*id));
                    ExpressionType::Bool
                }
            },
            Expression::Label(_, _) => {
                panic!(
                    "Labels must be expanded before transforming an expression into a stack-based expression"
                )
            }
            Expression::Function(name, args, _) => {
                if name.name == "min" || name.name == "max" {
                    let mut all_int = true;
                    for arg in args {
                        let arg_type = Self::process_expression(arg, operations, variable_manager);
                        match arg_type {
                            ExpressionType::Int => {}
                            ExpressionType::Bool => {
                                panic!("Cannot apply {} to boolean operands", name.name)
                            }
                            ExpressionType::Float => {
                                all_int = false;
                            }
                        }
                    }
                    if name.name == "min" {
                        if all_int {
                            operations.push(Operation::MaxInt(args.len()));
                            ExpressionType::Int
                        } else {
                            operations.push(Operation::MaxFloat(args.len()));
                            ExpressionType::Float
                        }
                    } else {
                        if all_int {
                            operations.push(Operation::MinInt(args.len()));
                            ExpressionType::Int
                        } else {
                            operations.push(Operation::MinFloat(args.len()));
                            ExpressionType::Float
                        }
                    }
                } else if name.name == "floor" {
                    assert_eq!(args.len(), 1, "Function floor takes exactly one operand");
                    let arg_type = Self::process_expression(&args[0], operations, variable_manager);

                    if arg_type == ExpressionType::Float {
                        operations.push(Operation::Floor);
                        ExpressionType::Int
                    } else {
                        panic!("Function floor can only operate on floats");
                    }
                } else if name.name == "ceil" {
                    assert_eq!(args.len(), 1, "Function ceil takes exactly one operand");
                    let arg_type = Self::process_expression(&args[0], operations, variable_manager);

                    if arg_type == ExpressionType::Float {
                        operations.push(Operation::Ceil);
                        ExpressionType::Int
                    } else {
                        panic!("Function ceil can only operate on floats");
                    }
                } else if name.name == "round" {
                    assert_eq!(args.len(), 1, "Function round takes exactly one operand");
                    let arg_type = Self::process_expression(&args[0], operations, variable_manager);

                    if arg_type == ExpressionType::Float {
                        operations.push(Operation::Round);
                        ExpressionType::Int
                    } else {
                        panic!("Function round can only operate on floats");
                    }
                } else if name.name == "pow" {
                    assert_eq!(args.len(), 2, "Function pow takes exactly two operands");
                    let arg1_type =
                        Self::process_expression(&args[0], operations, variable_manager);

                    let mut arg2ops = Vec::new();
                    let arg2_type =
                        Self::process_expression(&args[0], &mut arg2ops, variable_manager);

                    if arg1_type == ExpressionType::Float || arg2_type == ExpressionType::Float {
                        if arg1_type == ExpressionType::Int {
                            operations.push(Operation::IntToFloat);
                        }
                        operations.append(&mut arg2ops);
                        if arg2_type == ExpressionType::Int {
                            operations.push(Operation::IntToFloat);
                        }
                        operations.push(Operation::PowFloat);
                        ExpressionType::Float
                    } else {
                        operations.append(&mut arg2ops);
                        operations.push(Operation::PowInt);
                        ExpressionType::Int
                    }
                } else if name.name == "mod" {
                    assert_eq!(args.len(), 2, "Function mod takes exactly two operands");
                    let arg1_type =
                        Self::process_expression(&args[0], operations, variable_manager);
                    let arg2_type =
                        Self::process_expression(&args[1], operations, variable_manager);

                    if arg1_type == ExpressionType::Int && arg2_type == ExpressionType::Int {
                        operations.push(Operation::Mod);
                        ExpressionType::Int
                    } else {
                        panic!("Function mod operates on two integer operands");
                    }
                } else if name.name == "log" {
                    assert_eq!(args.len(), 2, "Function log takes exactly two operands");
                    let arg1_type =
                        Self::process_expression(&args[0], operations, variable_manager);
                    if arg1_type == ExpressionType::Int {
                        operations.push(Operation::IntToFloat);
                    }
                    let arg2_type =
                        Self::process_expression(&args[1], operations, variable_manager);
                    if arg2_type == ExpressionType::Int {
                        operations.push(Operation::IntToFloat);
                    }

                    operations.push(Operation::LogFloat);
                    ExpressionType::Float
                } else {
                    panic!("Unknown function name {}", name.name);
                }
            }
            Expression::Minus(inner, _) => {
                let inner_type = Self::process_expression(inner, operations, variable_manager);
                match inner_type {
                    ExpressionType::Int => {
                        operations.push(Operation::NegateInt);
                    }
                    ExpressionType::Float => {
                        operations.push(Operation::NegateFloat);
                    }
                    ExpressionType::Bool => {
                        panic!("Cannot apply the unary minus to a boolean")
                    }
                }
                inner_type
            }
            Expression::Multiplication(arg1, arg2, _) => Self::int_or_float_operation(
                arg1,
                arg2,
                Operation::MultiplyInt,
                Operation::MultiplyFloat,
                operations,
                variable_manager,
            ),
            Expression::Division(arg1, arg2, _) => Self::int_or_float_operation(
                arg1,
                arg2,
                Operation::DivideInt,
                Operation::DivideFloat,
                operations,
                variable_manager,
            ),
            Expression::Addition(arg1, arg2, _) => Self::int_or_float_operation(
                arg1,
                arg2,
                Operation::AddInt,
                Operation::AddFloat,
                operations,
                variable_manager,
            ),
            Expression::Subtraction(arg1, arg2, _) => Self::int_or_float_operation(
                arg1,
                arg2,
                Operation::SubtractInt,
                Operation::SubtractFloat,
                operations,
                variable_manager,
            ),
            Expression::LessThan(arg1, arg2, _) => Self::int_or_float_operation(
                arg1,
                arg2,
                Operation::LessThanInt,
                Operation::LessThanFloat,
                operations,
                variable_manager,
            ),
            Expression::LessOrEqual(arg1, arg2, _) => Self::int_or_float_operation(
                arg1,
                arg2,
                Operation::LessOrEqualInt,
                Operation::LessOrEqualFloat,
                operations,
                variable_manager,
            ),
            Expression::GreaterThan(arg1, arg2, _) => Self::int_or_float_operation(
                arg1,
                arg2,
                Operation::GreaterThanInt,
                Operation::GreaterThanFloat,
                operations,
                variable_manager,
            ),
            Expression::GreaterOrEqual(arg1, arg2, _) => Self::int_or_float_operation(
                arg1,
                arg2,
                Operation::GreaterOrEqualInt,
                Operation::GreaterOrEqualFloat,
                operations,
                variable_manager,
            ),
            Expression::Equals(arg1, arg2, _) => Self::int_float_or_bool_operation(
                arg1,
                arg2,
                Operation::EqualsInt,
                Operation::EqualsFloat,
                Operation::EqualsBool,
                operations,
                variable_manager,
            ),
            Expression::NotEquals(arg1, arg2, _) => Self::int_float_or_bool_operation(
                arg1,
                arg2,
                Operation::NotEqualsInt,
                Operation::NotEqualsFloat,
                Operation::NotEqualsBool,
                operations,
                variable_manager,
            ),
            Expression::Negation(arg, _) => {
                let inner_type = Self::process_expression(arg, operations, variable_manager);
                if inner_type != ExpressionType::Bool {
                    panic!("Invalid type for the negation operator");
                }
                operations.push(Operation::NegateBool);
                ExpressionType::Bool
            }
            Expression::Conjunction(arg1, arg2, _) => {
                let type1 = Self::process_expression(arg1, operations, variable_manager);
                let type2 = Self::process_expression(arg2, operations, variable_manager);
                if type1 == ExpressionType::Bool && type2 == ExpressionType::Bool {
                    operations.push(Operation::Conjunction);
                    ExpressionType::Bool
                } else {
                    panic!("Conjunction can only operate on two booleans");
                }
            }
            Expression::Disjunction(arg1, arg2, _) => {
                let type1 = Self::process_expression(arg1, operations, variable_manager);
                let type2 = Self::process_expression(arg2, operations, variable_manager);
                if type1 == ExpressionType::Bool && type2 == ExpressionType::Bool {
                    operations.push(Operation::Disjunction);
                    ExpressionType::Bool
                } else {
                    panic!("Conjunction can only operate on two booleans");
                }
            }
            Expression::IfAndOnlyIf(arg1, arg2, _) => {
                let type1 = Self::process_expression(arg1, operations, variable_manager);
                let type2 = Self::process_expression(arg2, operations, variable_manager);
                if type1 == ExpressionType::Bool && type2 == ExpressionType::Bool {
                    operations.push(Operation::IfAndOnlyIf);
                    ExpressionType::Bool
                } else {
                    panic!("Conjunction can only operate on two booleans");
                }
            }
            Expression::Implies(arg1, arg2, _) => {
                let type1 = Self::process_expression(arg1, operations, variable_manager);
                let type2 = Self::process_expression(arg2, operations, variable_manager);
                if type1 == ExpressionType::Bool && type2 == ExpressionType::Bool {
                    operations.push(Operation::Implies);
                    ExpressionType::Bool
                } else {
                    panic!("Conjunction can only operate on two booleans");
                }
            }
            Expression::Ternary(guard, arg1, arg2, _) => {
                let guard_type = Self::process_expression(guard, operations, variable_manager);
                let type1 = Self::process_expression(arg1, operations, variable_manager);
                let mut ops2 = Vec::new();
                let type2 = Self::process_expression(arg2, &mut ops2, variable_manager);
                if guard_type == ExpressionType::Bool {
                    if type1 == ExpressionType::Int && type2 == ExpressionType::Float {
                        operations.push(Operation::IntToFloat);
                    }
                    operations.append(&mut ops2);

                    if type1 == ExpressionType::Float && type2 == ExpressionType::Int {
                        operations.push(Operation::IntToFloat);
                    }

                    if type1 == ExpressionType::Int && type2 == ExpressionType::Int {
                        operations.push(Operation::TernaryInt);
                        ExpressionType::Int
                    } else if (type1 == ExpressionType::Int || type1 == ExpressionType::Float)
                        || (type2 == ExpressionType::Int && type2 == ExpressionType::Float)
                    {
                        operations.push(Operation::TernaryFloat);
                        ExpressionType::Float
                    } else if type1 == ExpressionType::Bool && type2 == ExpressionType::Bool {
                        operations.push(Operation::TernaryBool);
                        ExpressionType::Bool
                    } else {
                        panic!("Incompatible operands for ternary operation")
                    }
                } else {
                    panic!("The guard of a ternary expression must be a boolean");
                }
            }
        }
    }

    fn int_or_float_operation<S: Clone>(
        arg1: &Expression<VariableReference, S>,
        arg2: &Expression<VariableReference, S>,
        int_operation: Operation<VariableReference>,
        float_operation: Operation<VariableReference>,
        operations: &mut Vec<Operation<VariableReference>>,
        variable_manager: &VariableManager<VariableReference, S>,
    ) -> ExpressionType {
        let mut ops2 = Vec::new();
        let type1 = Self::process_expression(arg1, operations, variable_manager);
        let type2 = Self::process_expression(arg2, &mut ops2, variable_manager);

        if type1 == ExpressionType::Int && type2 == ExpressionType::Float {
            operations.push(Operation::IntToFloat);
        }
        operations.append(&mut ops2);
        if type1 == ExpressionType::Float && type2 == ExpressionType::Int {
            operations.push(Operation::IntToFloat);
        }
        if type1 == ExpressionType::Float || type2 == ExpressionType::Float {
            operations.push(float_operation);
            ExpressionType::Float
        } else if type1 == ExpressionType::Int && type2 == ExpressionType::Int {
            operations.push(int_operation);
            ExpressionType::Int
        } else {
            panic!("The operation can only operate on ints and floats");
        }
    }

    fn int_float_or_bool_operation<S: Clone>(
        arg1: &Expression<VariableReference, S>,
        arg2: &Expression<VariableReference, S>,
        int_operation: Operation<VariableReference>,
        float_operation: Operation<VariableReference>,
        bool_operation: Operation<VariableReference>,
        operations: &mut Vec<Operation<VariableReference>>,
        variable_manager: &VariableManager<VariableReference, S>,
    ) -> ExpressionType {
        let mut ops2 = Vec::new();
        let type1 = Self::process_expression(arg1, operations, variable_manager);
        let type2 = Self::process_expression(arg2, &mut ops2, variable_manager);

        if type1 == ExpressionType::Int && type2 == ExpressionType::Float {
            operations.push(Operation::IntToFloat);
        }
        operations.append(&mut ops2);
        if type1 == ExpressionType::Float && type2 == ExpressionType::Int {
            operations.push(Operation::IntToFloat);
        }
        if type1 == ExpressionType::Float || type2 == ExpressionType::Float {
            operations.push(float_operation);
            ExpressionType::Float
        } else if type1 == ExpressionType::Int && type2 == ExpressionType::Int {
            operations.push(int_operation);
            ExpressionType::Int
        } else if type1 == ExpressionType::Bool && type2 == ExpressionType::Bool {
            operations.push(bool_operation);
            ExpressionType::Bool
        } else {
            panic!(
                "The operation can only operate on two booleans or on a combination of integers and floats"
            );
        }
    }

    fn evaluate<VS: ValuationSource>(&self, valuations: &VS) {
        let mut stack = EvaluationStack::new();
        for operation in &self.operations {
            match operation {
                Operation::PushInt(i) => stack.ints.push(*i),
                Operation::PushFloat(f) => stack.floats.push(*f),
                Operation::PushBool(b) => stack.bools.push(*b),
                Operation::PushVarOrConstInt(id) => stack.ints.push(valuations.get_int(*id)),
                Operation::PushVarOrConstFloat(id) => stack.floats.push(valuations.get_float(*id)),
                Operation::PushVarOrConstBool(id) => stack.bools.push(valuations.get_bool(*id)),
                Operation::NegateInt => {
                    let len = stack.ints.len();
                    stack.ints[len - 1] *= -1;
                }
                Operation::NegateFloat => {
                    let len = stack.floats.len();
                    stack.floats[len - 1] *= -1.0;
                }
                Operation::IntToFloat => {
                    let i = stack.ints.pop().unwrap();
                    stack.floats.push(i as f64);
                }
                Operation::MultiplyInt => {
                    let a = stack.ints.pop().unwrap();
                    let b = stack.ints.pop().unwrap();
                    stack.ints.push(a * b);
                }
                Operation::MultiplyFloat => {
                    let a = stack.floats.pop().unwrap();
                    let b = stack.floats.pop().unwrap();
                    stack.floats.push(a * b);
                }
                Operation::DivideInt => {
                    let a = stack.ints.pop().unwrap();
                    let b = stack.ints.pop().unwrap();
                    stack.ints.push(a / b);
                }
                Operation::DivideFloat => {
                    let a = stack.floats.pop().unwrap();
                    let b = stack.floats.pop().unwrap();
                    stack.floats.push(a / b);
                }
                Operation::AddInt => {
                    let a = stack.ints.pop().unwrap();
                    let b = stack.ints.pop().unwrap();
                    stack.ints.push(a + b);
                }
                Operation::AddFloat => {
                    let a = stack.floats.pop().unwrap();
                    let b = stack.floats.pop().unwrap();
                    stack.floats.push(a + b);
                }
                Operation::SubtractInt => {
                    let a = stack.ints.pop().unwrap();
                    let b = stack.ints.pop().unwrap();
                    stack.ints.push(a - b);
                }
                Operation::SubtractFloat => {
                    let a = stack.floats.pop().unwrap();
                    let b = stack.floats.pop().unwrap();
                    stack.floats.push(a - b);
                }
                Operation::LessThanInt => {
                    let a = stack.ints.pop().unwrap();
                    let b = stack.ints.pop().unwrap();
                    stack.bools.push(a < b);
                }
                Operation::LessThanFloat => {
                    let a = stack.floats.pop().unwrap();
                    let b = stack.floats.pop().unwrap();
                    stack.bools.push(a < b);
                }
                Operation::LessOrEqualInt => {
                    let a = stack.ints.pop().unwrap();
                    let b = stack.ints.pop().unwrap();
                    stack.bools.push(a <= b);
                }
                Operation::LessOrEqualFloat => {
                    let a = stack.floats.pop().unwrap();
                    let b = stack.floats.pop().unwrap();
                    stack.bools.push(a <= b);
                }
                Operation::GreaterThanInt => {
                    let a = stack.ints.pop().unwrap();
                    let b = stack.ints.pop().unwrap();
                    stack.bools.push(a > b);
                }
                Operation::GreaterThanFloat => {
                    let a = stack.floats.pop().unwrap();
                    let b = stack.floats.pop().unwrap();
                    stack.bools.push(a > b);
                }
                Operation::GreaterOrEqualInt => {
                    let a = stack.ints.pop().unwrap();
                    let b = stack.ints.pop().unwrap();
                    stack.bools.push(a >= b);
                }
                Operation::GreaterOrEqualFloat => {
                    let a = stack.floats.pop().unwrap();
                    let b = stack.floats.pop().unwrap();
                    stack.bools.push(a >= b);
                }
                Operation::EqualsInt => {
                    let a = stack.ints.pop().unwrap();
                    let b = stack.ints.pop().unwrap();
                    stack.bools.push(a == b);
                }
                Operation::EqualsFloat => {
                    let a = stack.floats.pop().unwrap();
                    let b = stack.floats.pop().unwrap();
                    stack.bools.push(a == b);
                }
                Operation::EqualsBool => {
                    let a = stack.bools.pop().unwrap();
                    let b = stack.bools.pop().unwrap();
                    stack.bools.push(a == b);
                }
                Operation::NotEqualsInt => {
                    let a = stack.ints.pop().unwrap();
                    let b = stack.ints.pop().unwrap();
                    stack.bools.push(a != b);
                }
                Operation::NotEqualsFloat => {
                    let a = stack.floats.pop().unwrap();
                    let b = stack.floats.pop().unwrap();
                    stack.bools.push(a != b);
                }
                Operation::NotEqualsBool => {
                    let a = stack.bools.pop().unwrap();
                    let b = stack.bools.pop().unwrap();
                    stack.bools.push(a != b);
                }
                Operation::NegateBool => {
                    let len = stack.bools.len();
                    stack.bools[len - 1] = !stack.bools[len - 1];
                }
                Operation::Conjunction => {
                    let a = stack.bools.pop().unwrap();
                    let b = stack.bools.pop().unwrap();
                    stack.bools.push(a && b);
                }
                Operation::Disjunction => {
                    let a = stack.bools.pop().unwrap();
                    let b = stack.bools.pop().unwrap();
                    stack.bools.push(a || b);
                }
                Operation::IfAndOnlyIf => {
                    let a = stack.bools.pop().unwrap();
                    let b = stack.bools.pop().unwrap();
                    stack.bools.push(a == b);
                }
                Operation::Implies => {
                    let a = stack.bools.pop().unwrap();
                    let b = stack.bools.pop().unwrap();
                    stack.bools.push(!a || b);
                }
                Operation::TernaryInt => {
                    let g = stack.bools.pop().unwrap();
                    let a = stack.ints.pop().unwrap();
                    let b = stack.ints.pop().unwrap();
                    if g {
                        stack.ints.push(a);
                    } else {
                        stack.ints.push(b);
                    }
                }
                Operation::TernaryFloat => {
                    let g = stack.bools.pop().unwrap();
                    let a = stack.floats.pop().unwrap();
                    let b = stack.floats.pop().unwrap();
                    if g {
                        stack.floats.push(a);
                    } else {
                        stack.floats.push(b);
                    }
                }
                Operation::TernaryBool => {
                    let g = stack.bools.pop().unwrap();
                    let a = stack.bools.pop().unwrap();
                    let b = stack.bools.pop().unwrap();
                    if g {
                        stack.bools.push(a);
                    } else {
                        stack.bools.push(b);
                    }
                }
                Operation::MinInt(n) => {
                    let mut min = i64::MAX;
                    for _ in 0..*n {
                        let val = stack.ints.pop().unwrap();
                        min = min.min(val);
                    }
                    stack.ints.push(min);
                }
                Operation::MinFloat(n) => {
                    let mut min = f64::MAX;
                    for _ in 0..*n {
                        let val = stack.floats.pop().unwrap();
                        min = min.min(val);
                    }
                    stack.floats.push(min);
                }
                Operation::MaxInt(n) => {
                    let mut max = i64::MIN;
                    for _ in 0..*n {
                        let val = stack.ints.pop().unwrap();
                        max = max.max(val);
                    }
                    stack.ints.push(max);
                }
                Operation::MaxFloat(n) => {
                    let mut max = f64::MIN;
                    for _ in 0..*n {
                        let val = stack.floats.pop().unwrap();
                        max = max.max(val);
                    }
                    stack.floats.push(max);
                }
                Operation::Floor => {
                    let val = stack.floats.pop().unwrap();
                    stack.ints.push(val.floor() as i64);
                }
                Operation::Ceil => {
                    let val = stack.floats.pop().unwrap();
                    stack.ints.push(val.floor() as i64);
                }
                Operation::Round => {
                    let val = stack.floats.pop().unwrap();
                    let rounded = if val < 0.0 && val.fract() == 0.5 {
                        val.ceil()
                    } else {
                        val.round()
                    };
                    stack.ints.push(rounded as i64);
                }
                Operation::PowInt => {
                    let a = stack.ints.pop().unwrap();
                    let b = stack.ints.pop().unwrap();
                    stack.ints.push(a.pow(b as u32));
                }
                Operation::PowFloat => {
                    let a = stack.floats.pop().unwrap();
                    let b = stack.floats.pop().unwrap();
                    stack.floats.push(a.powf(b));
                }
                Operation::Mod => {
                    let a = stack.ints.pop().unwrap();
                    let b = stack.ints.pop().unwrap();
                    stack.ints.push(a.rem_euclid(b));
                }
                Operation::LogFloat => {
                    let a = stack.floats.pop().unwrap();
                    let b = stack.floats.pop().unwrap();
                    stack.floats.push(a.log(b));
                }
            }
        }
    }
}

pub struct EvaluationStack {
    ints: Vec<i64>,
    floats: Vec<f64>,
    bools: Vec<bool>,
}

impl EvaluationStack {
    pub fn new() -> Self {
        Self {
            ints: Vec::new(),
            floats: Vec::new(),
            bools: Vec::new(),
        }
    }
}

pub enum Operation<V> {
    PushInt(i64),
    PushFloat(f64),
    PushBool(bool),

    PushVarOrConstInt(V),
    PushVarOrConstFloat(V),
    PushVarOrConstBool(V),

    NegateInt,
    NegateFloat,

    IntToFloat,

    MultiplyInt,
    MultiplyFloat,
    DivideInt,
    DivideFloat,
    AddInt,
    AddFloat,
    SubtractInt,
    SubtractFloat,

    LessThanInt,
    LessThanFloat,
    LessOrEqualInt,
    LessOrEqualFloat,
    GreaterThanInt,
    GreaterThanFloat,
    GreaterOrEqualInt,
    GreaterOrEqualFloat,
    EqualsInt,
    EqualsFloat,
    EqualsBool,
    NotEqualsInt,
    NotEqualsFloat,
    NotEqualsBool,
    NegateBool,
    Conjunction,
    Disjunction,
    IfAndOnlyIf,
    Implies,
    TernaryInt,
    TernaryFloat,
    TernaryBool,

    MinInt(usize),
    MinFloat(usize),
    MaxInt(usize),
    MaxFloat(usize),
    Floor,
    Ceil,
    Round,
    PowInt,
    PowFloat,
    Mod,
    LogFloat,
}
