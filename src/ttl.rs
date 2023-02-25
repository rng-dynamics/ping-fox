#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Ttl(pub u8);

impl From<u8> for Ttl {
    fn from(integer: u8) -> Self {
        Ttl(integer)
    }
}

impl From<Ttl> for u8 {
    fn from(ttl: Ttl) -> Self {
        ttl.0
    }
}

impl std::fmt::Display for Ttl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fmt() {
        assert_eq!("8", format!("{}", Ttl(8)));
    }
}
