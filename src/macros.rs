#[macro_export]
macro_rules! expect_token {
    ($expr:expr, $msg:expr, $($pat:pat => $pat_expr:expr),* $(,)?) => {
        match $expr {
            $(
                Some(Ok($pat)) => Ok($pat_expr),
            )*
            Some(Err(err)) => Err(err.into()),
            _ => Err($crate::parser::ParseError::ExpectedToken($msg.into())),
        }
    };
}

#[macro_export]
macro_rules! operator_map {
    (
        $lhs:expr, $operator:expr, $rhs:expr,
        $($operator_kind:ident {
            $($lhs_kind:ident($lhs_ident:ident), $rhs_kind:ident($rhs_ident:ident) => $expr:expr)*
        })*
    ) => {
        match $operator {
            $(
                $crate::command::Operator::$operator_kind => match ($lhs, $rhs) {
                    $((Value::$lhs_kind($lhs_ident), Value::$rhs_kind($rhs_ident)) => Ok($expr),)*
                    (lhs, rhs) => Err(
                        terrors::OneOf::new(
                            $crate::database::CannotEvaluateError { lhs, operator: $operator, rhs }
                        )
                    ),
                },
            )*
        }
    };
}

#[macro_export]
macro_rules! operator {
    ($(#[$precedence:expr] $($ident:ident)|*),*) => {
        #[derive(Debug, Clone, Copy)]
        pub enum Operator {
            $($($ident,)*)*
        }

        impl Operator {
            fn precedence(&self) -> u8 {
                match self {
                    $($(Self::$ident)|* => $precedence,)*
                }
            }
        }
    };
}
