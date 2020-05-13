use std::{fmt::Debug, ops::Add, time::SystemTime};

//-- Resource --------------------------------------------------------------------------------------

/// A resource represents anything that is input or output of a build step.
///
/// Typical resources are files and directories with build steps like copying, moving, linking or
/// invoking external commands.
///
/// The main resource property is it's optional timestamp. Build steps should treat output resources
/// without one as out of date and build it unconditionally. When input resource does not have it's
/// timestamp it should be considered as changed and therefore rebuild the output the same way as
/// when input is newer the output. Typical scenario for lack or timestamp is when output resources
/// do not exists yet (clean builds)
///
pub trait Resource : Debug {

    /// Name of the resource used for logging and error reporting
    //fn name(&self) -> &str;

    /// Return resource timestamp. Can be None for input resources that should be considered as
    /// changed in every build run or output resources that do not exists yet.
    fn timestamp(&self) -> Option<SystemTime>;

    /// Build the resource form a given `src` resource as a side product of given function `by`
    /// respecting resource timestamps meaning that function `by` will only be ran if the output
    /// needs to be build.
    ///
    /// This method forces the `by` function to handle any errors on it's own and stop Cargo build
    /// using a panic. To propagate the error, use [`mk_from_result()`](#method.mk_from_result)
    ///
    //TODO: test
    fn mk_from<F, R, S>(&self, description: &str, src: S, by: F)
        where R:Resource, S:AsResource<R>, F: FnOnce() -> ()
    {
        let src = src.as_res();
        let target_time = self.timestamp();
        if target_time == None || src.timestamp() > target_time {
            println!("Building: {:?} from {:?}: {}", self, src, description);
            by();
        }
    }

    /// Same as [`mk_from()`](#method.mk_from) with error propagation
    //TODO: test
    fn mk_from_result<E, F, R, S>(&self, description: &str, src: S, by: F) -> Result<(), E>
        where R:Resource, S:AsRef<R>, F: FnOnce() -> Result<(), E>
    {
        let src = src.as_ref();
        let target_time = self.timestamp();
        if target_time == None || src.timestamp() > target_time {
            println!("Building: {:?} from {:?}: {}", self, src, description);
            return by()
        }

        Ok(())
    }
}

pub trait AsResource<R> {
    fn as_res(&self) -> &R;
}

impl<R> AsResource<R> for R where R:Resource {
    fn as_res(&self) -> &R {
        &self
    }
}

impl<R> AsResource<R> for &R where R:Resource {
    fn as_res(&self) -> &R {
        self
    }
}

//--- Resource for Vec -----------------------------------------------------------------------------

impl<R> Resource for Vec<R>
    where R:Resource
{
    fn timestamp(&self) -> Option<SystemTime> {
        timestamp(self.iter())
    }
}

//TODO: test
pub fn timestamp<T: AsResource<R>, R: Resource>(iter: impl Iterator<Item=T>) -> Option<SystemTime> {
    iter.fold(None, |result, entry| {
        let timestamp = entry.as_res().timestamp();
        if timestamp > result {
            return timestamp;
        }
        result
    })
}

//-- Set -------------------------------------------------------------------------------------------

/// Ordered list of owned resources
///
/// This is like Vec with '+' overloaded for easy adding of resources. Resources added by reference
/// are cloned.
///
#[derive(Debug,Clone)]
pub struct Set<T> {
    items: Vec<T>
}

impl<R> Add<&R> for Set<R> where R: Clone {
    type Output = Set<R>;

    fn add(mut self, rhs: &R) -> Self::Output {
        self.items.push(rhs.clone());
        self
    }
}

impl<R> Add<R> for Set<R> {
    type Output = Set<R>;

    fn add(mut self, rhs: R) -> Self::Output {
        self.items.push(rhs);
        self
    }
}

impl<T> AsRef<Set<T>> for Set<T> {
    fn as_ref(&self) -> &Set<T> {
        self
    }
}

impl<T> From<Vec<T>> for Set<T> {
    fn from(val: Vec<T>) -> Self {
        Set { items: val }
    }
}

impl<T> IntoIterator for Set<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl<R> Resource for Set<R> where R:Resource {

    fn timestamp(&self) -> Option<SystemTime> {
        self.items.timestamp()
    }
}