pub(crate) struct Ttl(u8);

impl From<u8> for Ttl {
    fn from(integer: u8) -> Self {
        Ttl(integer)
    }
}
