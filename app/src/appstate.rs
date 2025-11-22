use std::sync::Arc;





////////////////////////////////////////////////////////////
/// Data that is not loaded, loading, or loaded. Designed for yew;
/// this means that data is considered equal iff it is stored
/// in the same position in memory
#[derive(Debug)]
pub enum AsyncData<T> {
    NotLoaded,
    Loading,
    Loaded(Arc<T>)
}
impl<T> AsyncData<T> {

    ////////////////////////////////////////////////////////////
    /// Wrap data as loaded AsyncData
    pub fn new(data: T) -> AsyncData<T> {
        AsyncData::Loaded(Arc::new(data))
    }

}




////////////////////////////////////////////////////////////
/// Ensure cloning just clones the Arc;
/// derive(Clone) adds overly restrictive requirements on T
impl<T> Clone for AsyncData<T> {
    fn clone(&self) -> Self {
        match self {
            AsyncData::Loaded(this) => {
                AsyncData::Loaded(this.clone())
            },
            AsyncData::NotLoaded => {
                AsyncData::NotLoaded
            },
            AsyncData::Loading => {
                AsyncData::Loading
            },
        }
    }
}




////////////////////////////////////////////////////////////
/// For yew - AsyncData is "equal" if pointers are the same. Otherwise assume the data changed.
/// This speeds up comparison
impl<T> PartialEq for AsyncData<T> {

    fn eq(&self, other: &Self) -> bool {
        match self {
            AsyncData::Loaded(this) => {
                match other {
                    AsyncData::Loaded(other) => Arc::ptr_eq(this,other),
                    _ => false
                }

            },
            AsyncData::NotLoaded => {
                match other {
                    AsyncData::NotLoaded => true,
                    _ => false
                }
            },
            AsyncData::Loading => {
                match other {
                    AsyncData::Loading => true,
                    _ => false
                }
            },
        }
    }

}

