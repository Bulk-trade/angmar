#[macro_export]
macro_rules! validate {
    ($assert:expr, $err:expr) => {{
        if ($assert) {
             Ok::<(), ProgramError>(())
        } else {
            let error = $err;
            msg!("Error {} thrown at {}:{}", error, file!(), line!());
            Err(error.into())
        }
    }};
    
    ($assert:expr, $err:expr, $($arg:tt)+) => {{
        if ($assert) {
             Ok::<(), ProgramError>(())
        } else {
            let error = $err;
            msg!("Error {} thrown at {}:{}", error, file!(), line!());
            msg!($($arg)*);
            Err(error.into())
        }
    }};
}