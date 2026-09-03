use std::sync::Arc;

////////////////////////////////////////////////////////////
/// Data that is not loaded, loading, or loaded. Designed for yew;
/// this means that data is considered equal iff it is stored
/// in the same position in memory
#[derive(Debug)]
pub enum AsyncData<T> {
    NotLoaded,
    Loading,
    Loaded(Arc<T>),
    /// The request finished but did not produce usable data: the server
    /// answered with an error status, was unreachable, or sent a body we
    /// could not parse. Holds a message fit to show the user.
    Failed(Arc<String>),
}
impl<T> AsyncData<T> {
    ////////////////////////////////////////////////////////////
    /// Wrap data as loaded AsyncData
    pub fn new(data: T) -> AsyncData<T> {
        AsyncData::Loaded(Arc::new(data))
    }

    ////////////////////////////////////////////////////////////
    /// Wrap an error message as failed AsyncData
    pub fn failed(msg: impl Into<String>) -> AsyncData<T> {
        AsyncData::Failed(Arc::new(msg.into()))
    }
}

////////////////////////////////////////////////////////////
/// Ensure cloning just clones the Arc;
/// derive(Clone) adds overly restrictive requirements on T
impl<T> Clone for AsyncData<T> {
    fn clone(&self) -> Self {
        match self {
            AsyncData::Loaded(this) => AsyncData::Loaded(this.clone()),
            AsyncData::NotLoaded => AsyncData::NotLoaded,
            AsyncData::Loading => AsyncData::Loading,
            AsyncData::Failed(msg) => AsyncData::Failed(msg.clone()),
        }
    }
}

////////////////////////////////////////////////////////////
/// For yew - AsyncData is "equal" if pointers are the same. Otherwise assume the data changed.
/// This speeds up comparison
impl<T> PartialEq for AsyncData<T> {
    fn eq(&self, other: &Self) -> bool {
        match self {
            AsyncData::Loaded(this) => match other {
                AsyncData::Loaded(other) => Arc::ptr_eq(this, other),
                _ => false,
            },
            AsyncData::NotLoaded => match other {
                AsyncData::NotLoaded => true,
                _ => false,
            },
            AsyncData::Loading => match other {
                AsyncData::Loading => true,
                _ => false,
            },
            AsyncData::Failed(this) => match other {
                AsyncData::Failed(other) => this == other,
                _ => false,
            },
        }
    }
}
