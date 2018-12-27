use std::fmt;
use std::iter;

use h5::types::{FixedAscii, FixedUnicode, VarLenArray, VarLenAscii, VarLenUnicode};
use h5::H5Type;
use hdf5_types::Array;

use ndarray::ArrayD;
use rand::distributions::{Alphanumeric, Uniform};
use rand::prelude::{Rng, SliceRandom};

pub fn gen_shape<R: Rng + ?Sized>(rng: &mut R, ndim: usize) -> Vec<usize> {
    iter::repeat(()).map(|_| rng.gen_range(0, 11)).take(ndim).collect()
}

pub fn gen_ascii<R: Rng + ?Sized>(rng: &mut R, len: usize) -> String {
    iter::repeat(()).map(|_| rng.sample(Alphanumeric)).take(len).collect()
}

pub trait Gen: Sized + fmt::Debug {
    fn gen<R: Rng + ?Sized>(rng: &mut R) -> Self;
}

macro_rules! impl_gen_primitive {
    ($ty:ty) => {
        impl Gen for $ty {
            fn gen<R: Rng + ?Sized>(rng: &mut R) -> Self {
                rng.gen()
            }
        }
    };
    ($ty:ty, $($tys:ty),+) => {
        impl_gen_primitive!($ty);
        impl_gen_primitive!($($tys),*);
    };
}

impl_gen_primitive!(usize, isize, u8, u16, u32, u64, i8, i16, i32, i64, bool, f32, f64);

pub fn gen_vec<R: Rng + ?Sized, T: Gen>(rng: &mut R, size: usize) -> Vec<T> {
    iter::repeat(()).map(|_| T::gen(rng)).take(size).collect()
}

pub fn gen_arr<T, R>(rng: &mut R, ndim: usize) -> ArrayD<T>
where
    T: H5Type + Gen,
    R: Rng + ?Sized,
{
    let shape = gen_shape(rng, ndim);
    let size = shape.iter().product();
    let vec = gen_vec(rng, size);
    ArrayD::from_shape_vec(shape, vec).unwrap()
}

impl<A: Array<Item = u8>> Gen for FixedAscii<A> {
    fn gen<R: Rng + ?Sized>(rng: &mut R) -> Self {
        let len = rng.sample(Uniform::new_inclusive(0, A::capacity()));
        let dist = Uniform::new_inclusive(0, 127);
        let mut v = Vec::with_capacity(len);
        for _ in 0..len {
            v.push(rng.sample(dist));
        }
        unsafe { FixedAscii::from_ascii_unchecked(&v) }
    }
}

impl<A: Array<Item = u8>> Gen for FixedUnicode<A> {
    fn gen<R: Rng + ?Sized>(rng: &mut R) -> Self {
        let len = rng.sample(Uniform::new_inclusive(0, A::capacity()));
        let mut s = String::new();
        for _ in 0..len {
            let c = rng.gen::<char>();
            if c != '\0' {
                if s.as_bytes().len() + c.len_utf8() >= len {
                    break;
                }
                s.push(c);
            }
        }
        unsafe { FixedUnicode::from_str_unchecked(s) }
    }
}

impl Gen for VarLenAscii {
    fn gen<R: Rng + ?Sized>(rng: &mut R) -> Self {
        let len = rng.sample(Uniform::new_inclusive(0, 8));
        let dist = Uniform::new_inclusive(0, 127);
        let mut v = Vec::with_capacity(len);
        for _ in 0..len {
            v.push(rng.sample(dist));
        }
        unsafe { VarLenAscii::from_ascii_unchecked(&v) }
    }
}

impl Gen for VarLenUnicode {
    fn gen<R: Rng + ?Sized>(rng: &mut R) -> Self {
        let len = rng.sample(Uniform::new_inclusive(0, 8));
        let mut s = String::new();
        while s.len() < len {
            let c = rng.gen::<char>();
            if c != '\0' {
                s.push(c);
            }
        }
        unsafe { VarLenUnicode::from_str_unchecked(s) }
    }
}

impl<T: Gen + Copy> Gen for VarLenArray<T> {
    fn gen<R: Rng + ?Sized>(rng: &mut R) -> Self {
        let len = rng.sample(Uniform::new_inclusive(0, 8));
        let mut v = Vec::with_capacity(len);
        for _ in 0..len {
            v.push(Gen::gen(rng));
        }
        VarLenArray::from_slice(&v)
    }
}

#[derive(H5Type, Clone, Copy, Debug, PartialEq)]
#[repr(i16)]
pub enum Enum {
    X = -2,
    Y = 3,
}

impl Gen for Enum {
    fn gen<R: Rng + ?Sized>(rng: &mut R) -> Self {
        *[Enum::X, Enum::Y].choose(rng).unwrap()
    }
}

#[derive(H5Type, Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct TupleStruct(bool, Enum);

impl Gen for TupleStruct {
    fn gen<R: Rng + ?Sized>(rng: &mut R) -> Self {
        TupleStruct(Gen::gen(rng), Gen::gen(rng))
    }
}

#[derive(H5Type, Clone, Debug, PartialEq)]
#[repr(C)]
pub struct FixedStruct {
    fa: FixedAscii<[u8; 3]>,
    fu: FixedUnicode<[u8; 11]>,
    tuple: (i8, u64, f32),
    array: [TupleStruct; 2],
}

impl Gen for FixedStruct {
    fn gen<R: Rng + ?Sized>(rng: &mut R) -> Self {
        FixedStruct {
            fa: Gen::gen(rng),
            fu: Gen::gen(rng),
            tuple: (Gen::gen(rng), Gen::gen(rng), Gen::gen(rng)),
            array: [Gen::gen(rng), Gen::gen(rng)],
        }
    }
}

#[derive(H5Type, Clone, Debug, PartialEq)]
#[repr(C)]
pub struct VarLenStruct {
    va: VarLenAscii,
    vu: VarLenUnicode,
    vla: VarLenArray<Enum>,
}

impl Gen for VarLenStruct {
    fn gen<R: Rng + ?Sized>(rng: &mut R) -> Self {
        VarLenStruct { va: Gen::gen(rng), vu: Gen::gen(rng), vla: Gen::gen(rng) }
    }
}
