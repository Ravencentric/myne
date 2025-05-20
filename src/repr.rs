pub(crate) trait PyRepr {
    /// Provides a string representation intended to mimic Python's built-in `__repr__` method.
    fn __repr__(&self) -> String;
}

impl PyRepr for String {
    fn __repr__(&self) -> String {
        format!("'{}'", &self.replace("'", "\\'"))
    }
}

impl PyRepr for bool {
    fn __repr__(&self) -> String {
        let pytrue = "True".to_string();
        let pyfalse = "False".to_string();
        if *self { pytrue } else { pyfalse }
    }
}

impl PyRepr for u8 {
    fn __repr__(&self) -> String {
        self.to_string()
    }
}

impl PyRepr for u16 {
    fn __repr__(&self) -> String {
        self.to_string()
    }
}

impl<T: PyRepr> PyRepr for Option<T> {
    fn __repr__(&self) -> String {
        match self {
            Some(value) => value.__repr__(),
            None => "None".to_string(),
        }
    }
}
