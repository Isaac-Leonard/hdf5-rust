use ffi::h5::hsize_t;
use ffi::h5i::{H5I_DATASPACE, hid_t};
use ffi::h5s::{H5S_UNLIMITED, H5Sget_simple_extent_dims, H5Sget_simple_extent_ndims,
               H5Screate_simple};

use error::Result;
use handle::{Handle, ID, get_id_type};
use object::Object;

use std::ptr;
use libc::c_int;

pub type Ix = usize;

pub trait Dimension {
    fn ndim(&self) -> usize;
    fn dims(&self) -> Vec<Ix>;

    fn size(&self) -> Ix {
        let dims = self.dims();
        if dims.is_empty() { 0 } else { dims.iter().fold(1, |acc, &el| acc * el) }
    }
}

impl<'a, T: Dimension> Dimension for &'a T {
    fn ndim(&self) -> usize { Dimension::ndim(*self) }
    fn dims(&self) -> Vec<Ix> { Dimension::dims(*self) }
}

impl Dimension for Vec<Ix> {
    fn ndim(&self) -> usize { self.len() }
    fn dims(&self) -> Vec<Ix> { self.clone() }
}

impl Dimension for () {
    fn ndim(&self) -> usize { 0 }
    fn dims(&self) -> Vec<Ix> { vec![] }
}

impl Dimension for Ix {
    fn ndim(&self) -> usize { 1 }
    fn dims(&self) -> Vec<Ix> { vec![*self] }
}

impl Dimension for (Ix,) {
    fn ndim(&self) -> usize { 1 }
    fn dims(&self) -> Vec<Ix> { vec![self.0] }
}

impl Dimension for (Ix, Ix) {
    fn ndim(&self) -> usize { 2 }
    fn dims(&self) -> Vec<Ix> { vec![self.0, self.1] }
}

pub struct Dataspace {
    handle: Handle,
}

impl Dataspace {
    pub fn new<D: Dimension>(d: D) -> Result<Dataspace> {
        let rank = d.ndim();
        let mut dims: Vec<hsize_t> = vec![];
        let mut max_dims: Vec<hsize_t> = vec![];
        for dim in d.dims().iter() {
            dims.push(*dim as hsize_t);
            max_dims.push(H5S_UNLIMITED);
        }
        Dataspace::from_id(h5try!(H5Screate_simple(rank as c_int, dims.as_ptr(),
                                                   max_dims.as_ptr())))
    }
}

impl Dimension for Dataspace {
    fn ndim(&self) -> usize {
        h5call!(H5Sget_simple_extent_ndims(self.id())).unwrap_or(0) as usize
    }

    fn dims(&self) -> Vec<Ix> {
        let ndim = self.ndim();
        if ndim > 0 {
            let mut dims: Vec<hsize_t> = Vec::with_capacity(ndim);
            unsafe { dims.set_len(ndim); }
            if h5call!(H5Sget_simple_extent_dims(self.id(), dims.as_mut_ptr(),
                                                 ptr::null_mut())).is_ok() {
                return dims.iter().cloned().map(|x| x as usize).collect();
            }
        }
        vec![]
    }
}

impl ID for Dataspace {
    fn id(&self) -> hid_t {
        self.handle.id()
    }

    fn from_id(id: hid_t) -> Result<Dataspace> {
        match get_id_type(id) {
            H5I_DATASPACE => Ok(Dataspace { handle: try!(Handle::new(id)) }),
            _             => Err(From::from(format!("Invalid dataspace id: {}", id))),
        }
    }
}

impl Object for Dataspace {}

#[cfg(test)]
mod tests {
    use super::{Dimension, Ix};

    #[test]
    pub fn test_dimension() {
        fn f<D: Dimension>(d: D) -> (usize, Vec<Ix>, Ix) { (d.ndim(), d.dims(), d.size()) }

        assert_eq!(f(()), (0, vec![], 0));
        assert_eq!(f(&()), (0, vec![], 0));
        assert_eq!(f(2), (1, vec![2], 2));
        assert_eq!(f(&3), (1, vec![3], 3));
        assert_eq!(f((4,)), (1, vec![4], 4));
        assert_eq!(f(&(5,)), (1, vec![5], 5));
        assert_eq!(f((1, 2)), (2, vec![1, 2], 2));
        assert_eq!(f(&(3, 4)), (2, vec![3, 4], 12));
        assert_eq!(f(vec![2, 3]), (2, vec![2, 3], 6));
        assert_eq!(f(&vec![4, 5]), (2, vec![4, 5], 20));
    }
}
