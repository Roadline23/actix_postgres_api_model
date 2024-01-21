use validator::ValidationError;

pub trait IsEmpty {
    fn is_empty(&self) -> bool;
}

macro_rules! impl_is_empty {
    ($($t:ty),*) => {
        $(
            impl IsEmpty for $t {
                fn is_empty(&self) -> bool {
                    self.is_empty()
                }
            }
        )*
    };
}

impl_is_empty!(String, Vec<String>);

pub fn required<T: IsEmpty>(value: &T) -> Result<(), ValidationError> {
    if value.is_empty() {
        Err(ValidationError::new("required"))
    } else {
        Ok(())
    }
}

pub fn not_a_spe(spe: &Vec<String>) -> Result<(), ValidationError> {
    let specialities = vec![
        "Pilates",
        "Psychology",
        "Osteopathy",
        "physiotherapy",
    ];

    for s in spe {
        if !specialities.contains(&s.as_str()) {
            return Err(ValidationError::new("not_a_spe"));
        }
    }
    Ok(())
}

pub fn must_accept(terms: &bool) -> Result<(), ValidationError> {
    if !terms {
        return Err(ValidationError::new("must_accept"));
    }
    Ok(())
}

pub fn has_errors(vector: Vec<&String>) -> bool {
    return vector.iter().any(|v| !v.is_empty());
}