macro_rules! catch_err {
    ($($e:tt)*) => {
        match (std::panic::catch_unwind(|| -> VdrResult<_> {$($e)*})) {
            Ok(Ok(a)) => a,
            Ok(Err(err)) => { // vdr error
                let code = ErrorCode::from(&err);
                set_last_error(Some(err));
                code
            }
            Err(_) => { // panic error
                let error = (VdrErrorKind::Unexpected, "Panic during execution").into();
                let code = ErrorCode::from(&error);
                set_last_error(Some(error));
                code
            }
        }
    }
}

macro_rules! check_useful_c_ptr {
    ($e:expr) => {
        if ($e).is_null() {
            return Err(input_err("Invalid pointer for result value"));
        }
    };
}

macro_rules! slice_from_c_ptr {
    ($bytes:expr, $len:expr) => {{
        if ($bytes).is_null() {
            Err(input_err("Invalid pointer for input value"))
        } else if ($len) <= 0 {
            Err(input_err("Buffer size must be greater than zero"))
        } else {
            Ok(unsafe { std::slice::from_raw_parts($bytes, $len) })
        }
    }};
}

macro_rules! read_lock {
    ($e:expr) => {
        ($e).read().map_err(|err| {
            err_msg(
                VdrErrorKind::Unexpected,
                format!("Error acquiring read lock: {}", err),
            )
        })
    };
}

macro_rules! write_lock {
    ($e:expr) => {
        ($e).write().map_err(|err| {
            err_msg(
                VdrErrorKind::Unexpected,
                format!("Error acquiring write lock: {}", err),
            )
        })
    };
}
