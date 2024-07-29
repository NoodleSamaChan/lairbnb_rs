#[derive(Debug)]
pub struct LairLon(f64);

impl LairLon {
    /// Returns an instance of `SubscriberName` if the input satisfies all
    /// our validation constraints on subscriber names.  
    /// It panics otherwise.
    pub fn parse(s: f64) -> Result<LairLon, String> {
        let is_a_float = s.is_nan();

        if is_a_float {
            Err(format!("{} is not a valid lon value.", s))
        } else {
            Ok(Self(s))
        }
    }
}

impl AsRef<f64> for LairLon {
    fn as_ref(&self) -> &f64 {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::LairLon;
    use claims::assert_ok;

    #[test]
    fn a_valid_lon_is_parsed_successfully() {
        let lon = 1.1;
        assert_ok!(LairLon::parse(lon));
    }
}