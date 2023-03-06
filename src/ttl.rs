// TODO: should we do that also with a trait and an associated type?
// Not sure. This actually seems simple enough.
// Nevertheless, try what is shorter and nicer.
// KISS

type TtlInnerType = u8;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Ttl(pub TtlInnerType);

impl From<TtlInnerType> for Ttl {
    fn from(integer: TtlInnerType) -> Self {
        Ttl(integer)
    }
}

impl From<Ttl> for TtlInnerType {
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
