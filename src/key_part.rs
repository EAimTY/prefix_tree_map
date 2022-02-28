use crate::std_lib::{Debug, FmtResult, Formatter, Ordering};

/// A part of a key
#[derive(Clone, Eq, PartialEq)]
pub enum KeyPart<E, W> {
    Exact(E),
    Wildcard(W),
}

impl<E, W> KeyPart<E, W> {
    /// Convert a `&KeyPart<E, W>` to a `KeyPart<&E, &W>`
    pub fn as_ref(&self) -> KeyPart<&E, &W> {
        match self {
            KeyPart::Exact(key) => KeyPart::Exact(key),
            KeyPart::Wildcard(key) => KeyPart::Wildcard(key),
        }
    }

    /// Is the key part a `KeyPart::Wildcard`
    pub fn is_wildcard(&self) -> bool {
        matches!(self, KeyPart::Wildcard(_))
    }

    /// Is the key part an `KeyPart::Exact`
    pub fn is_exact(&self) -> bool {
        matches!(self, KeyPart::Exact(_))
    }

    pub(crate) fn unwrap_wildcard(self) -> W {
        match self {
            KeyPart::Wildcard(key) => key,
            KeyPart::Exact(_) => unreachable!(),
        }
    }
}

impl<E: Debug, W: Debug> Debug for KeyPart<E, W> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            KeyPart::Exact(key) => write!(f, "Exact({:?})", key),
            KeyPart::Wildcard(key) => write!(f, "Wildcard({:?})", key),
        }
    }
}

impl<E, W> PartialOrd for KeyPart<E, W>
where
    E: Clone + Ord,
    W: Clone + Ord,
{
    fn partial_cmp(&self, other: &KeyPart<E, W>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<E, W> Ord for KeyPart<E, W>
where
    E: Clone + Ord,
    W: Clone + Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (KeyPart::Exact(key_part), KeyPart::Exact(other_key_part)) => {
                key_part.cmp(other_key_part)
            }
            (KeyPart::Wildcard(key_part), KeyPart::Wildcard(other_key_part)) => {
                key_part.cmp(other_key_part)
            }
            (KeyPart::Exact(_), KeyPart::Wildcard(_)) => Ordering::Greater,
            (KeyPart::Wildcard(_), KeyPart::Exact(_)) => Ordering::Less,
        }
    }
}
