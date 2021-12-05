use std::{cmp::Ordering, fmt::Debug};

/// A part of a key.
#[derive(Clone, Eq, PartialEq)]
pub enum KeyPart<E, W> {
    Exact(E),
    Wildcard(W),
}

impl<E, W> KeyPart<E, W> {
    /// Convert from `&KeyPart<E, W>` to `KeyPart<&E, &W>`.
    pub fn as_ref(&self) -> KeyPart<&E, &W> {
        match self {
            KeyPart::Exact(key) => KeyPart::Exact(key),
            KeyPart::Wildcard(key) => KeyPart::Wildcard(key),
        }
    }

    /// Return true if the key part is a wildcard.
    pub fn is_wildcard(&self) -> bool {
        matches!(self, KeyPart::Wildcard(_))
    }

    /// Return true if the key part is a exact key.
    pub fn is_exact(&self) -> bool {
        matches!(self, KeyPart::Exact(_))
    }

    pub(crate) fn unwrap_wildcard(self) -> W {
        if let KeyPart::Wildcard(key) = self {
            key
        } else {
            panic!();
        }
    }
}

impl<E: Debug, W: Debug> Debug for KeyPart<E, W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
