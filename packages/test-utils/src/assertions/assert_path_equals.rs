#[macro_export]
macro_rules! assert_path_equals {
    ($left:expr, $right:expr $(,)?) => {{
        $crate::assert_path_equals!($left, $right, "");
    }};
    ($left:expr, $right:expr, $($msg:tt)+) => {{
        use std::path::Path;

        let left_expr = stringify!($left);
        let right_expr = stringify!($right);

        let left_val = $left;
        let right_val = $right;

        let left = Path::new(&left_val).canonicalize().unwrap_or_else(|e| {
            panic!("assert_path_equals!: {left_expr} ({left_val:?}) canonicalize failed: {e}\n{}", format_args!($($msg)+));
        });
        let right = Path::new(&right_val).canonicalize().unwrap_or_else(|e| {
            panic!("assert_path_equals!: {right_expr} ({right_val:?}) canonicalize failed: {e}\n{}", format_args!($($msg)+));
        });

        if left != right {
            panic!(
                "assert_path_equals! failed:\n  left:  {left_expr} => {left:?}\n  right: {right_expr} => {right:?}\n{}",
                format_args!($($msg)+),
            );
        }
    }};
}
