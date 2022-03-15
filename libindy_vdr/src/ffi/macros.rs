macro_rules! catch_err {
    ($($e:tt)*) => {
        match std::panic::catch_unwind(|| -> VdrResult<_> {$($e)*}) {
            Ok(Ok(a)) => a,
            Ok(Err(err)) => { // vdr error
                let code = ErrorCode::from(err.kind());
                set_last_error(Some(err));
                code
            }
            Err(_) => { // panic error
                let err: VdrError = (VdrErrorKind::Unexpected, "Panic during execution").into();
                let code = ErrorCode::from(err.kind());
                set_last_error(Some(err));
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
