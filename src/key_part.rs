use std::cmp::Ordering;

#[derive(Clone, Eq, PartialEq)]
pub enum KeyPart<E, W> {
    Exact(E),
    Wildcard(W),
}

impl<E, W> KeyPart<E, W> {
    pub fn as_ref(&self) -> KeyPart<&E, &W> {
        match self {
            KeyPart::Exact(key) => KeyPart::Exact(key),
            KeyPart::Wildcard(key) => KeyPart::Wildcard(key),
        }
    }

    pub fn is_wildcard(&self) -> bool {
        matches!(self, KeyPart::Wildcard(_))
    }

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

impl<E, W> PartialOrd for KeyPart<E, W>
where
    E: Clone + Ord + PartialEq,
    W: Clone + Ord + PartialEq,
{
    fn partial_cmp(&self, other: &KeyPart<E, W>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<E, W> Ord for KeyPart<E, W>
where
    E: Clone + Ord + PartialEq,
    W: Clone + Ord + PartialEq,
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
