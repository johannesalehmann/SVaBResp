use crate::expressions::stack_based_expressions::StackBasedExpression;
use crate::expressions::{TreeWalkingEvaluator, ValuationSource, VariableType};
use paste::paste;
use prism_model::{Expression, Identifier, VariableManager, VariableReference};

struct MockValueSource {}

impl MockValueSource {
    fn panic<T>(&self) -> T {
        panic!("A mock value source cannot provide any types or values")
    }
}

impl ValuationSource for MockValueSource {
    fn get_int(&self, index: VariableReference) -> i64 {
        let _ = index;
        self.panic()
    }

    fn get_bool(&self, index: VariableReference) -> bool {
        let _ = index;
        self.panic()
    }

    fn get_float(&self, index: VariableReference) -> f64 {
        let _ = index;
        self.panic()
    }

    fn get_type(&self, index: VariableReference) -> VariableType {
        let _ = index;
        self.panic()
    }
}

macro_rules! test_expr {
    ($name:ident, $ex:expr, $expected_type:ident, $result:expr) => {
        paste! {
        #[test]
        fn [<$name _tree_walking_evaluator>]() {
            let expression = $ex;
            let value = TreeWalkingEvaluator::new().[<evaluate_as_ $expected_type>](&expression, &MockValueSource {});
            assert_eq!(
                value, $result,
                "Expression value does not match expected result"
            );
        }

        #[test]
        fn [<$name _stack_based_expression>]() {
            let expression = $ex;
            let variable_manager: VariableManager<StackBasedExpression<VariableReference>, ()> = VariableManager::new();
            let stack_based_expression = StackBasedExpression::from_expression(&expression, &variable_manager);
            let value = stack_based_expression.[<evaluate_as_ $expected_type>](&MockValueSource {});
            assert_eq!(
                value, $result,
                "Expression value does not match expected result"
            );
        }
        }
    };
}

fn ident(name: &'static str) -> Identifier<()> {
    Identifier::new_potentially_reserved(name, ()).unwrap()
}

fn int(int: i64) -> Expression<VariableReference, ()> {
    Expression::int(int)
}
fn float(float: f64) -> Expression<VariableReference, ()> {
    Expression::float(float)
}
fn bool(bool: bool) -> Expression<VariableReference, ()> {
    Expression::bool(bool)
}

test_expr!(int, int(20), int, 20);
test_expr!(int_negative, int(-12345), int, -12345);
test_expr!(int_zero, int(0), int, 0);
test_expr!(int_as_float, int(543), float, 543.0);
test_expr!(int_as_float_negative, int(-789), float, -789.0);

test_expr!(float, float(1.2345), float, 1.2345);
test_expr!(float_negative, float(-8.123), float, -8.123);
test_expr!(float_zero, float(0.0), float, 0.0);

test_expr!(bool_true, bool(true), bool, true);
test_expr!(bool_false, bool(false), bool, false);

// TODO: Evaluate variable valuations

macro_rules! test_min_max {
    ($name:ident, int, $smaller_arg:expr, $larger_arg:expr, $min_result:expr, $max_result:expr) => {
        paste! {
            test_expr!(
                [<$name _min_1>],
                Expression::function(ident("min"), &[$smaller_arg, $larger_arg]),
                int,
                $min_result
            );
            test_expr!(
                [<$name _min_2>],
                Expression::function(ident("min"), &[$larger_arg, $smaller_arg]),
                int,
                $min_result
            );
            test_expr!(
                [<$name _max_1>],
                Expression::function(ident("max"), &[$smaller_arg, $larger_arg]),
                int,
                $max_result
            );
            test_expr!(
                [<$name _max_2>],
                Expression::function(ident("max"), &[$larger_arg, $smaller_arg]),
                int,
                $max_result
            );
            test_min_max!([<$name _as_float>], float, $smaller_arg, $larger_arg, $min_result as f64, $max_result as f64);
        }
    };
    ($name:ident, float, $smaller_arg:expr, $larger_arg:expr, $min_result:expr, $max_result:expr) => {
        paste! {
            test_expr!(
                [<$name _min_1>],
                Expression::function(ident("min"), &[$smaller_arg, $larger_arg]),
                float,
                $min_result
            );
            test_expr!(
                [<$name _min_2>],
                Expression::function(ident("min"), &[$larger_arg, $smaller_arg]),
                float,
                $min_result
            );
            test_expr!(
                [<$name _max_1>],
                Expression::function(ident("max"), &[$smaller_arg, $larger_arg]),
                float,
                $max_result
            );
            test_expr!(
                [<$name _max_2>],
                Expression::function(ident("max"), &[$larger_arg, $smaller_arg]),
                float,
                $max_result
            );
        }
    };
    ($name:ident, int, [$($arg:expr),*], $min_result:expr, $max_result:expr) => {
        paste! {
            test_expr!(
                [<$name _min>],
                Expression::function(ident("min"), &[$($arg),*]),
                int,
                $min_result
            );
            test_expr!(
                [<$name _max>],
                Expression::function(ident("max"), &[$($arg),*]),
                int,
                $max_result
            );
            test_min_max!([<$name as_float>], float, [$($arg),*], $min_result as f64, $max_result as f64);
        }
    };
    ($name:ident, float, [$($arg:expr),*], $min_result:expr, $max_result:expr) => {
        paste! {
            test_expr!(
                [<$name _min>],
                Expression::function(ident("min"), &[$($arg),*]),
                float,
                $min_result
            );
            test_expr!(
                [<$name _max>],
                Expression::function(ident("max"), &[$($arg),*]),
                float,
                $max_result
            );
        }
    };
}

test_min_max!(int_both_positive, int, int(13), int(32), 13, 32);
test_min_max!(int_equal, int, int(1234), int(1234), 1234, 1234);
test_min_max!(int_negative, int, int(-32), int(12), -32, 12);
test_min_max!(float_and_int_1, float, float(-32.0), int(12), -32.0, 12.0);
test_min_max!(float_and_int_2, float, int(123), float(123.4), 123.0, 123.4);
test_min_max!(
    multi_int,
    int,
    [int(-123), int(-12), int(0), int(32)],
    -123,
    32
);
test_min_max!(three_int, int, [int(12), int(34), int(34)], 12, 34);
test_min_max!(
    multi_mixed_ints,
    float,
    [int(123), float(12.5), int(23)],
    12.5,
    123.0
);
test_min_max!(
    multi_mixed_ints_2,
    float,
    [float(74.3), float(12.5), int(23), int(17)],
    12.5,
    74.3
);
test_min_max!(
    multi_mixed_ints_complex,
    float,
    [
        float(74.3).minus(int(12)),
        float(12.5).times(int(1).divide_by(int(2))),
        int(23).plus(int(5)),
        int(17).minus(int(1))
    ],
    6.25,
    62.3
);

test_min_max!(
    multi_float,
    float,
    [float(-10.6), float(-15.98), float(5.123), float(1.5)],
    -15.98,
    5.123
);

macro_rules! test_rounding {
    ($name:ident, $operand:expr, $floored:expr, $ceiled: expr, $rounded:expr) => {
        paste! {
            test_expr!(
                [<$name _floor>],
                Expression::function(ident("floor"), &[$operand]),
                int,
                $floored
            );
            test_expr!(
                [<$name _ceil>],
                Expression::function(ident("ceil"), &[$operand]),
                int,
                $ceiled
            );
            test_expr!(
                [<$name _round>],
                Expression::function(ident("round"), &[$operand]),
                int,
                $rounded
            );
        }
    };
}

test_rounding!(integer, int(17), 17, 17, 17);
test_rounding!(zero, float(0.0), 0, 0, 0);
test_rounding!(five_point_two, float(5.2), 5, 6, 5);
test_rounding!(five_point_five, float(5.5), 5, 6, 6);
test_rounding!(five_point_eight, float(5.8), 5, 6, 6);
test_rounding!(negative_25_3, float(-25.3), -26, -25, -25);
test_rounding!(negative_25_5, float(-25.5), -26, -25, -25);
test_rounding!(negative_25_8, float(-25.8), -26, -25, -26);
test_rounding!(negative_0_5, float(-0.5), -1, 0, 0);

test_expr!(
    pow_2_4,
    Expression::function(ident("pow"), [int(2), int(4)]),
    int,
    16
);
test_expr!(
    pow_3_0,
    Expression::function(ident("pow"), [int(3), int(0)]),
    int,
    1
);
test_expr!(
    pow_float_16_0_0_5,
    Expression::function(ident("pow"), [float(16.0), float(0.5)]),
    float,
    4.0
);
test_expr!(
    pow_float_int,
    Expression::function(ident("pow"), [float(1.5), int(2)]),
    float,
    2.25
);
test_expr!(
    pow_int_float,
    Expression::function(ident("pow"), [int(4), float(1.5)]),
    float,
    8.0
);

test_expr!(
    mod_3_1,
    Expression::function(ident("mod"), [int(3), int(1)]),
    int,
    0
);
test_expr!(
    mod_negative_3_1,
    Expression::function(ident("mod"), [int(-3), int(1)]),
    int,
    0
);
test_expr!(
    mod_27_5,
    Expression::function(ident("mod"), [int(27), int(5)]),
    int,
    2
);
test_expr!(
    mod_negative_27_5,
    Expression::function(ident("mod"), [int(-27), int(5)]),
    int,
    3
);
test_expr!(
    mod_negative_121_11,
    Expression::function(ident("mod"), [int(121), int(11)]),
    int,
    0
);

test_expr!(
    log_int,
    Expression::function(ident("log"), [int(100), int(10)]),
    float,
    2.0
);
test_expr!(
    log_int_float,
    Expression::function(ident("log"), [int(81), float(3.0)]),
    float,
    4.0
);
test_expr!(
    log_float_int,
    Expression::function(ident("log"), [float(10.0), int(100)]),
    float,
    0.5
);
test_expr!(
    log_float,
    Expression::function(ident("log"), [float(1.5), float(2.25)]),
    float,
    0.5
);

test_expr!(minus_int, int(4).negate_value(), int, -4);
test_expr!(minus_int_negative, int(-10).negate_value(), int, 10);
test_expr!(minus_int_zero, int(0).negate_value(), int, 0);
test_expr!(
    minus_int_double_negation,
    int(17).negate_value().negate_value(),
    int,
    17
);

test_expr!(minus_float, float(3.23).negate_value(), float, -3.23);
test_expr!(minus_float_negative, float(-5.7).negate_value(), float, 5.7);
test_expr!(minus_float_zero, float(0.0).negate_value(), float, 0.0);
test_expr!(
    minus_float_triple_negation,
    float(-123.456).negate_value().negate_value().negate_value(),
    float,
    123.456
);

test_expr!(multiply_int, int(3).times(int(5)), int, 15);
test_expr!(multiply_int_zero, int(5).times(int(0)), int, 0);
test_expr!(multiply_int_negative, int(3).times(int(-6)), int, -18);
test_expr!(
    multiply_int_negated,
    int(4).negate_value().times(int(5)),
    int,
    -20
);
test_expr!(
    multiply_int_nested,
    int(3).times(int(-6).times(int(2))),
    int,
    -36
);

test_expr!(multiply_float, float(3.5).times(float(1.5)), float, 5.25);
test_expr!(
    multiply_float_zero,
    float(5.0).times(float(0.0)),
    float,
    0.0
);
test_expr!(
    multiply_float_negative,
    float(3.0).times(float(-6.0)),
    float,
    -18.0
);
test_expr!(
    multiply_float_negated,
    float(4.0).negate_value().times(float(5.0)),
    float,
    -20.0
);
test_expr!(
    multiply_float_nested,
    int(3).times(int(-6).times(float(2.5))),
    float,
    -45.0
);
test_expr!(multiply_float_int, float(2.5).times(int(3)), float, 7.5);
test_expr!(
    multiply_int_float,
    int(-5).times(float(-0.3).negate_value()),
    float,
    -1.5
);

test_expr!(divide_int, int(20).divide_by(int(-4)), float, -5.0);
test_expr!(divide_int_2, int(3).divide_by(int(-5)), float, -0.6);
test_expr!(divide_int_float, int(11).divide_by(float(0.5)), float, 22.0);
test_expr!(divide_float_int, float(5.5).divide_by(int(2)), float, 2.75);
test_expr!(
    divide_floats,
    float(-5.5).divide_by(float(0.1)),
    float,
    -55.0
);
test_expr!(
    divide_nested,
    int(150)
        .divide_by(int(3).divide_by(int(2)))
        .divide_by(int(1).divide_by(float(0.675))),
    float,
    67.5
);

test_expr!(add_ints, int(10).plus(int(11)), int, 21);
test_expr!(add_ints_as_float, int(10).plus(int(11)), float, 21.0);
test_expr!(add_negative_ints, int(-15).plus(int(-7)), int, -22);
test_expr!(add_zero, int(8).plus(int(0)), int, 8);
test_expr!(add_int_float, int(10).plus(float(5.5)), float, 15.5);
test_expr!(
    add_int_float_negated,
    int(-10).negate_value().plus(float(5.5).negate_value()),
    float,
    4.5
);
test_expr!(add_float_int, float(16.8).plus(int(-8)), float, 8.8);
test_expr!(
    add_float_int_nested,
    float(16.8)
        .plus(int(3))
        .plus(int(-8).plus(int(3)).plus(float(5.1))),
    float,
    19.9
);
test_expr!(add_floats, float(7.3).plus(float(2.6)), float, 9.9);
test_expr!(
    add_floats_negative,
    float(5.2).negate_value().plus(float(-1.1).negate_value()),
    float,
    -4.1
);

test_expr!(subtract_ints, int(12).minus(int(6)), int, 6);
test_expr!(subtract_ints_to_negative, int(12).minus(int(19)), int, -7);
test_expr!(
    subtract_ints_nested,
    int(12)
        .minus(int(3).negate_value())
        .minus(int(-5).negate_value().minus(int(7))),
    int,
    17
);
test_expr!(subtract_int_float, int(12).minus(float(4.5)), float, 7.5);
test_expr!(subtract_float_int, float(-5.8).minus(int(-2)), float, -3.8);
test_expr!(
    subtract_float_int_nested,
    float(-5.8).minus(float(3.1)).minus(int(-2).minus(int(1))),
    float,
    -5.9
);
test_expr!(subtract_floats, float(3.6).minus(float(5.2)), float, -1.6);
test_expr!(
    subtract_floats_nested,
    float(3.5)
        .minus(float(1.1))
        .minus(float(5.1).minus(float(5.2).negate_value())),
    float,
    -7.9
);

macro_rules! comparison {
    ($name:ident, ($op1:expr) lt ($op2:expr)) => {
        paste! {
            test_expr!([<$name _less_than>], $op1.less_than($op2), bool, true);
            test_expr!([<$name _greater_than>], $op1.greater_than($op2), bool, false);
            test_expr!([<$name _less_than_swapped>], $op2.less_than($op1), bool, false);
            test_expr!([<$name _greater_than_swapped>], $op2.greater_than($op1), bool, true);

            test_expr!([<$name _less_or_equal>], $op1.less_or_equal($op2), bool, true);
            test_expr!([<$name _greater_or_equal>], $op1.greater_or_equal($op2), bool, false);
            test_expr!([<$name _less_or_equal_swapped>], $op2.less_or_equal($op1), bool, false);
            test_expr!([<$name _greater_or_equal_swapped>], $op2.greater_or_equal($op1), bool, true);

            test_expr!([<$name _equals>], $op1.equals_to($op2), bool, false);
            test_expr!([<$name _not_equals>], $op1.not_equals_to($op2), bool, true);
            test_expr!([<$name _equals_swapped>], $op2.equals_to($op1), bool, false);
            test_expr!([<$name _not_equals_swapped>], $op2.not_equals_to($op1), bool, true);
        }
    };
    ($name:ident, ($op1:expr) eq ($op2:expr)) => {
        paste! {
            test_expr!([<$name _less_than>], $op1.less_than($op2), bool, false);
            test_expr!([<$name _greater_than>], $op1.greater_than($op2), bool, false);
            test_expr!([<$name _less_than_swapped>], $op2.less_than($op1), bool, false);
            test_expr!([<$name _greater_than_swapped>], $op2.greater_than($op1), bool, false);

            test_expr!([<$name _less_or_equal>], $op1.less_or_equal($op2), bool, true);
            test_expr!([<$name _greater_or_equal>], $op1.greater_or_equal($op2), bool, true);
            test_expr!([<$name _less_or_equal_swapped>], $op2.less_or_equal($op1), bool, true);
            test_expr!([<$name _greater_or_equal_swapped>], $op2.greater_or_equal($op1), bool, true);

            test_expr!([<$name _equals>], $op1.equals_to($op2), bool, true);
            test_expr!([<$name _not_equals>], $op1.not_equals_to($op2), bool, false);
            test_expr!([<$name _equals_swapped>], $op2.equals_to($op1), bool, true);
            test_expr!([<$name _not_equals_swapped>], $op2.not_equals_to($op1), bool, false);
        }
    };
}

comparison!(different_ints, (int(4)) lt (int(6)));
comparison!(different_ints_negative, (int(-6)) lt (int(4)));
comparison!(different_ints_zero, (int(-3)) lt (int(0)));
comparison!(different_ints_complex, (int(-6).plus(int(3)).negate_value()) lt (int(4).times(int(-2).negate_value())));
comparison!(same_ints, (int(5)) eq (int(5)));
comparison!(same_ints_complex, (int(5).negate_value().plus(int(10))) eq (int(10).minus(int(5).negate_value().times(int(-1)))));

comparison!(different_int_float, (float(3.7)) lt (int(4)));
comparison!(different_int_float_negative, (float(-3.7)) lt (int(-3)));
comparison!(different_int_float_complex, (int(3).negate_value().times(float(1.5).negate_value())) lt (int(-3).times(int(2).negate_value())));
comparison!(same_float_int, (float(3.0)) eq  (int(3)));
comparison!(same_float_int_complex, (float(3.7).plus(int(3).divide_by(int(10)))) eq (int(2).times(int(1).plus(int(1)))));

test_expr!(negate_true, bool(true).negate_bool(), bool, false);
test_expr!(negate_false, bool(false).negate_bool(), bool, true);
test_expr!(
    negate_double,
    bool(true).negate_bool().negate_bool(),
    bool,
    true
);
test_expr!(
    negate_complex_true,
    (int(3).equals_to(float(1.5).times(int(2)))).negate_bool(),
    bool,
    false
);
test_expr!(
    negate_complex_false,
    (int(3).equals_to(float(1.6).times(int(2)))).negate_bool(),
    bool,
    true
);

macro_rules! test_boolean {
    ($name:ident, $op1:expr, $op2:expr, $value_conjunction:expr, $value_disjunction:expr, $value_if_and_only_if:expr, $value_implies:expr) => {
        paste! {
            test_expr!([<$name _conjunction >], $op1.and($op2), bool, $value_conjunction);
            test_expr!([<$name _disjunction >], $op1.or($op2), bool, $value_disjunction);
            test_expr!([<$name _if_and_only_if >], $op1.if_and_only_if($op2), bool, $value_if_and_only_if);
            test_expr!([<$name _implies >], $op1.implies($op2), bool, $value_implies);
        }
    };
}
test_boolean!(
    false_false,
    bool(false),
    bool(false),
    false,
    false,
    true,
    true
);
test_boolean!(
    false_true,
    bool(false),
    bool(true),
    false,
    true,
    false,
    true
);
test_boolean!(
    true_false,
    bool(true),
    bool(false),
    false,
    true,
    false,
    false
);
test_boolean!(true_true, bool(true), bool(true), true, true, true, true);

test_expr!(
    ternary_int_first_branch,
    bool(true).ternary(int(5), int(8)),
    int,
    5
);
test_expr!(
    ternary_int_second_branch,
    bool(false).ternary(int(5), int(8)),
    int,
    8
);
test_expr!(
    ternary_int_float_first_branch,
    bool(true).ternary(int(5), float(8.0)),
    float,
    5.0
);
test_expr!(
    ternary_int_float_second_branch,
    bool(false).ternary(int(5), float(8.0)),
    float,
    8.0
);
test_expr!(
    ternary_float_int_first_branch,
    bool(true).ternary(float(5.0), int(8)),
    float,
    5.0
);
test_expr!(
    ternary_float_int_second_branch,
    bool(false).ternary(float(5.0), int(8)),
    float,
    8.0
);
test_expr!(
    ternary_floats_first_branch,
    bool(true).ternary(float(5.5), float(8.5)),
    float,
    5.5
);
test_expr!(
    ternary_floats_second_branch,
    bool(false).ternary(float(5.5), float(8.5)),
    float,
    8.5
);
test_expr!(
    ternary_bool_first_branch,
    bool(true).ternary(bool(false), bool(true)),
    bool,
    false
);
test_expr!(
    ternary_bool_second_branch,
    bool(false).ternary(bool(false), bool(true)),
    bool,
    true
);
test_expr!(
    ternary_complex_float_first_branch,
    int(5)
        .equals_to(float(5.5).minus(float(0.5)))
        .ternary(int(3).plus(int(6)), int(17).divide_by(int(2))),
    float,
    9.0
);

test_expr!(
    ternary_complex_float_second_branch,
    int(5)
        .not_equals_to(float(5.5).minus(float(0.5)))
        .ternary(int(3).plus(int(6)), int(17).divide_by(int(2))),
    float,
    8.5
);
