#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct SequenceNumber(pub u16);

impl From<u16> for SequenceNumber {
    fn from(integer: u16) -> Self {
        SequenceNumber(integer)
    }
}

impl From<SequenceNumber> for u16 {
    fn from(sequence_number: SequenceNumber) -> Self {
        sequence_number.0
    }
}
