macro_rules! scanifc_try {
    ($expr:expr) => ({
        use {Error, last_error};
        let result = unsafe { $expr };
        if result != 0 {
            let msg = last_error()?;
            return Err(Error::Scanifc(result, msg));
        }
    })
}
