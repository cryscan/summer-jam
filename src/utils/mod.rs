mod collide;
mod damp;
mod interpolation;

pub use collide::collide_continuous;
pub use damp::Damp;
pub use interpolation::Interpolation;

pub fn merge_result<T, E>(first: Result<T, E>, second: Result<T, E>) -> Result<(T, T), E> {
    Ok((first?, second?))
}
