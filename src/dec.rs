#[derive(Copy, Debug, PartialEq, PartialOrd)]
pub struct Decimal {
    // TODO implement a real decimal type
    v: f64,
}

impl std::convert::TryFrom<&str> for Decimal {
    type Error = ();

    fn try_from(v: &str) -> Result<Self, Self::Error> {
        match v.parse() {
            Ok(v) => Ok(Decimal { v }),
            Err(_) => Err(()),
        }
    }
}

impl Clone for Decimal {
    fn clone(&self) -> Decimal {
        Decimal { v: self.v }
    }
}

impl std::fmt::Display for Decimal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match f.precision() {
            Some(p) => write!(f, "{1:.*}", p, self.v),
            None => write!(f, "{}", self.v),
        }
    }
}

impl core::ops::Add for Decimal {
    type Output = Decimal;
    fn add(self, o: Decimal) -> Self::Output {
        Decimal { v: self.v + o.v }
    }
}

impl core::ops::Sub for Decimal {
    type Output = Decimal;
    fn sub(self, o: Decimal) -> Self::Output {
        Decimal { v: self.v - o.v }
    }
}

impl core::ops::AddAssign for Decimal {
    fn add_assign(&mut self, o: Decimal) {
        self.v += o.v
    }
}

impl core::ops::Mul for Decimal {
    type Output = Decimal;
    fn mul(self, o: Decimal) -> Self::Output {
        Decimal { v: self.v * o.v }
    }
}

impl core::ops::Div for Decimal {
    type Output = Decimal;
    fn div(self, o: Decimal) -> Self::Output {
        Decimal { v: self.v / o.v }
    }
}
