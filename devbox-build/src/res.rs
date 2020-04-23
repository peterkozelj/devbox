//use super::File;

////////////////////////////////////////////////////////////////////////////////////////////////////
// Resource
////////////////////////////////////////////////////////////////////////////////////////////////////

pub trait Resource {
    fn timestamp(&self) -> Option<std::time::SystemTime>;
}

pub trait MkFrom<R:Resource, S:AsRef<R>, F:FnOnce() -> ()> {
    fn mk_from(&self, description: &str, src: S, by: F);
}

impl<T:Resource, R:Resource, S:AsRef<R>, F:FnOnce() -> ()> MkFrom<R, S, F> for T {

    //TODO: test
    fn mk_from(&self, description: &str, src: S, by: F) {
        let target_time = self.timestamp();
        if target_time == None || src.as_ref().timestamp() > target_time {
            println!("Building: {}", description);
            by()
        }
    }
}

impl<I:Resource, J:Iterator<Item=I>, T:IntoIterator<Item=I, IntoIter=J> + Clone> Resource for T {

    //TODO: test
    fn timestamp(&self) -> Option<std::time::SystemTime> {
        self.clone().into_iter().fold(None, |result, entry| {
            let timestamp = entry.timestamp();
            if timestamp > result {
                return timestamp;
            }
            result
        })
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Set
////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug,Clone)]
pub struct Set<T> { 
    pub items: Vec<T> //TODO: replace with HashSet or BTreSet
}

impl<T> AsRef<Set<T>> for Set<T> {
    fn as_ref(&self) -> &Set<T> {
        self
    }
}

impl<T> IntoIterator for Set<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}