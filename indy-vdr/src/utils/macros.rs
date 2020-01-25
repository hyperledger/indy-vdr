macro_rules! _map_err {
    ($lvl:expr, $expr:expr) => {
        |err| {
            log!($lvl, "{} - {}", $expr, err);
            err
        }
    };
    ($lvl:expr) => {
        |err| {
            log!($lvl, "{}", err);
            err
        }
    };
}

#[macro_export]
macro_rules! map_err_debug {
    () => ( _map_err!(::log::Level::Debug) );
    ($($arg:tt)*) => ( _map_err!(::log::Level::Debug, $($arg)*) )
}

macro_rules! unwrap_opt_or_return {
    ($opt:expr, $err:expr) => {
        match $opt {
            Some(val) => val,
            None => return $err,
        };
    };
}

macro_rules! unwrap_or_return {
    ($result:expr, $err:expr) => {
        match $result {
            Ok(res) => res,
            Err(_) => return $err,
        };
    };
}
