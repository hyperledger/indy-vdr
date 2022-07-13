#![allow(unused_macros)]

macro_rules! format_noop {
    ($($arg:tt)+) => {
        {
            // avoid unused variable warnings
            let _ = format_args!($($arg)+);
        }
    };
}

#[cfg(not(feature = "log"))]
macro_rules! log {
    ($($arg:tt)+) => {
        format_noop!($($arg)+)
    };
}
#[cfg(not(feature = "log"))]
macro_rules! error {
    ($($arg:tt)+) => {
        format_noop!($($arg)+)
    };
}
#[cfg(not(feature = "log"))]
macro_rules! warn {
    ($($arg:tt)+) => {
        format_noop!($($arg)+)
    };
}
#[cfg(not(feature = "log"))]
macro_rules! debug {
    ($($arg:tt)+) => {
        format_noop!($($arg)+)
    };
}
#[cfg(not(feature = "log"))]
macro_rules! info {
    ($($arg:tt)+) => {
        format_noop!($($arg)+)
    };
}
#[cfg(not(feature = "log"))]
macro_rules! trace {
    ($($arg:tt)+) => {
        format_noop!($($arg)+)
    };
}

macro_rules! map_err_log {
    (level: $lvl:tt, $($arg:tt)+) => {
        |err| {
            $lvl!($($arg)+, err);
            err
        }
    };
    ($($arg:tt)+) => {
        |err| {
            log!($($arg)+, err);
            err
        }
    };
}

macro_rules! unwrap_opt_or_return {
    ($opt:expr, $err:expr) => {
        match $opt {
            Some(val) => val,
            None => return $err,
        }
    };
}

macro_rules! unwrap_or_return {
    ($result:expr, $err:expr) => {
        match $result {
            Ok(res) => res,
            Err(_) => return $err,
        }
    };
}

macro_rules! unwrap_or_map_return {
    ($result:expr, $on_err:expr) => {
        match $result {
            Ok(res) => res,
            Err(err) =>
            {
                #[allow(clippy::redundant_closure_call)]
                return ($on_err)(err)
            }
        }
    };
}

macro_rules! in_closure {
    ($($e:tt)*) => {(|| -> Result<_, _> {$($e)*})()}
}
