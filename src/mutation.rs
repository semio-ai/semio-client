#[derive(Eq, PartialEq)]
pub enum Mutation<T> {
  None,
  Unset,
  Set(T),
}

impl<T> Mutation<T> {

  pub fn next(self, current: Option<T>) -> Option<T> {
    match self {
      Mutation::None => current,
      Mutation::Unset => None,
      Mutation::Set(t) => Some(t),
    }
  }
}

impl<T: PartialEq> Mutation<T> {
  pub fn union(self, other: Mutation<T>) -> Mutation<T> {
    if other == Self::None {
      self
    } else {
      other
    }
  }
}
