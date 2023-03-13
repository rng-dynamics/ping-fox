type SequenceNumberInnerType = u16;
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct SequenceNumber(pub SequenceNumberInnerType);

impl SequenceNumber {
    pub(crate) fn start_value() -> SequenceNumberInnerType {
        // ICMPv4 sequence numbers start from 1.
        SequenceNumberInnerType::from(1u8)
    }

    pub(crate) fn start_value2() -> SequenceNumber {
        SequenceNumber(Self::start_value())
    }

    pub(crate) fn max_value() -> SequenceNumberInnerType {
        SequenceNumberInnerType::max_value()
    }

    pub(crate) fn next(self) -> Self {
        if self.0 == Self::max_value() {
            Self::start_value2()
        } else {
            SequenceNumber(self.0 + 1)
        }
    }
}

impl From<SequenceNumber> for SequenceNumberInnerType {
    fn from(value: SequenceNumber) -> Self {
        value.0
    }
}

impl From<SequenceNumberInnerType> for SequenceNumber {
    fn from(value: SequenceNumberInnerType) -> Self {
        SequenceNumber(value)
    }
}
