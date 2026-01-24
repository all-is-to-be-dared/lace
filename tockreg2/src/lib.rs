#![no_std]
#![feature(never_type)]

// key idea: need to separate a few different concepts of 'register'
//  1. the method with interacting with the underlying 'hardware', which we will call the
//     [`Accessor`]
//  2. the register's 'interpretation' (i.e. the significance that we assign to its bit patterns),
//     which I call its ['Identity'] here
//  3. the general description of the register (its representation and read/write interpretations),
//     which is represented by the [`Register`] trait

pub mod accessor;
pub mod aliased;
pub mod access;

use core::fmt::Debug;
use core::ops::{BitAnd, BitOr, BitXor, Not, Shl, Shr};

/// An (uninhabitable) type that represents the interpretation of the bits in a register.
pub trait RegisterIdentity {}

// Useful implementation for when we don't wish to interpret the bits in a register as anything
// other than bits.
impl RegisterIdentity for () {}

pub trait UIntLike: Copy + Eq + BitOr + BitAnd + BitXor + Not + Shr + Shl + Debug {
    fn zero() -> Self;
}
macro_rules! impl_uintlike_for {
    ($($t:ty),*) => {
        $( impl $crate::UIntLike for $t { fn zero() -> Self { 0 } } )*
    }
}
impl_uintlike_for!(u8, u16, u32, u64, u128, usize);

pub trait Register {
    /// An architecturally suitable type to represent bit patterns of this register to the hardware.
    ///
    /// In other words, if you had a `*mut T` to an MMIO register, then `Repr` would be that `T`.
    type Repr: UIntLike;

    /// The interpretation of the bits of the register when being read.
    type ReadIdentity: RegisterIdentity;
    /// The interpretation of the bits of the register when being written to.
    type WriteIdentity: RegisterIdentity;
}

#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
pub mod compile_tests {
    use core::marker::PhantomData;
    use crate::{Register};
    use crate::access::{Safe, NoAccess, Access};
    use crate::accessor::{Accessor, Read, Write};
    use crate::accessor::dynamic_register::DynamicReg;
    use crate::accessor::static_register::StaticReg;
    use crate::aliased::Aliased;

    pub mod Control {
        use crate::*;

        pub struct Register;
        impl RegisterIdentity for Register {}
        impl crate::Register for Register {
            type Repr = u32;
            type ReadIdentity = Self;
            type WriteIdentity = Self;
        }
    }
    pub mod IoRead {
        use crate::*;
        pub struct Register(!);
        impl RegisterIdentity for Register {}
        impl crate::Register for Register {
            type Repr = u32;
            type ReadIdentity = Self;
            type WriteIdentity = Self;
        }
    }
    pub mod IoWrite {
        use crate::*;
        pub struct Register(!);
        impl RegisterIdentity for Register {}
        impl crate::Register for Register {
            type Repr = u32;
            type ReadIdentity = Self;
            type WriteIdentity = Self;
        }
    }

    pub trait Peri {
        type control<'s> : Accessor<Register = Control::Register> + Read where Self: 's;
        fn control(&self) -> Self::control<'_>;

        type io<'s> : Accessor<Register = Aliased<IoRead::Register, IoWrite::Register>> + Read + Write where Self: 's;
        fn io(&self) -> Self::io<'_>;
    }

    pub struct DynPeri(*mut u32);
    impl Peri for DynPeri {
        type control<'s> = DynamicReg<Control::Register, Safe, NoAccess>;

        fn control(&self) -> Self::control<'_> {
            unsafe { DynamicReg::summon_from_ptr(self.0.byte_add(4)) }
        }

        type io<'s> = DynamicReg<Aliased<IoRead::Register, IoWrite::Register>, Safe, Safe>;
        fn io(&self) -> Self::io<'_> {
            unsafe { DynamicReg::summon_from_ptr(self.0.byte_add(0)) }
        }
    }

    pub struct StaticPeri<const ADDR: usize>(());
    impl<const ADDR: usize> StaticPeri<ADDR> {
        /// # Safety
        ///
        /// `ADDR` must be the address of an instance of this peripheral's MMIO block.
        pub const unsafe fn summon() -> Self {
            const {
                #[allow(clippy::identity_op)]
                let addr = ADDR + 0;
                assert!(
                    addr.is_multiple_of(::core::alloc::Layout::new::<u32>().align()),
                    "register 'control' in StaticPeri would not be properly aligned"
                );
            }
            const {
                #[allow(clippy::identity_op)]
                let addr = ADDR + 4;
                assert!(
                    addr.is_multiple_of(::core::alloc::Layout::new::<u32>().align()),
                    "register 'io' in StaticPeri would not be properly aligned"
                );
            };
            Self(())
        }
    }
    impl<const ADDR: usize> Peri for StaticPeri<ADDR>
    {
        type control<'s> = StaticReg<Control::Register, Safe, NoAccess, ADDR, 4>;
        fn control(&self) -> Self::control<'_> {
            unsafe { StaticReg::summon() } }
        type io<'s> = StaticReg<Aliased<IoRead::Register, IoWrite::Register>, Safe, Safe, ADDR, 0>;
        fn io(&self) -> Self::io<'_> {
            unsafe { StaticReg::summon() } }
    }

    pub trait Delegate {
        fn read<Reg: Register>(&self, offset: usize) -> Reg::Repr;
        fn write<Reg: Register>(&self, offset: usize, v: Reg::Repr);
    }
    enum DelegateField {
        control,
        io,
    }
    pub struct DelegateRegister<'del, Reg: Register, R: Access, W: Access, D: Delegate> {
        delegate: &'del D,
        offset: usize,
        _pd: PhantomData<(Reg, R, W)>,
    }
    impl <'del, Reg: Register, R: Access, W: Access, D: Delegate> Accessor for DelegateRegister<'del, Reg, R, W, D> {
        type Register = Reg;
    }
    impl<'del, Reg: Register, W: Access, D: Delegate> Read for DelegateRegister<'del, Reg, Safe, W, D> {
        fn read(&self) -> <Self::Register as Register>::Repr {
            self.delegate.read::<Self::Register>(self.offset)
        }
    }
    impl<'del, Reg: Register, R: Access, D: Delegate> Write for DelegateRegister<'del, Reg, R, Safe, D> {
        fn write(&self, v: <Self::Register as Register>::Repr) {
            self.delegate.write::<Self::Register>(self.offset, v)
        }
    }
    pub struct FakePeri<D: Delegate>(D);
    impl<D: Delegate + 'static> Peri for FakePeri<D> {
        type control<'s> = DelegateRegister<'s, Control::Register, Safe, NoAccess, D>;

        fn control(&self) -> Self::control<'_> {
            Self::control { delegate: &self.0, offset: 4, _pd: PhantomData }
        }

        type io<'s> = DelegateRegister<'s, Aliased<IoRead::Register, IoWrite::Register>, Safe, Safe, D>;

        fn io(&self) -> Self::io<'_> {
            Self::io { delegate: &self.0, offset: 0, _pd: PhantomData }
        }
    }

    #[test]
    pub fn test_peri_test() {
        let dp = DynPeri(0xdead_beef_0000usize as *mut _);
        peri_test(dp);
        // this should fail to compile:
        let sp = unsafe { StaticPeri::<0xdead_beef_1001usize>::summon() };
        peri_test(sp);
    }

    pub fn peri_test<P: Peri>(peri: P) -> u32 {
        peri.io().write(peri.io().read());
        peri.control().read()
    }

}
