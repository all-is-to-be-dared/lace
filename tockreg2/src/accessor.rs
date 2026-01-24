use crate::Register;

pub trait Accessor {
    type Register: Register;
}

pub trait Read: Accessor {
    fn read(&self) -> <Self::Register as Register>::Repr;
}
pub trait Write: Accessor {
    fn write(&self, value: <Self::Register as Register>::Repr);
}
pub trait UnsafeRead: Accessor {
    /// # Safety
    /// Reading this register has hardware-specific safety requirements which the caller must comply
    /// with.
    unsafe fn read(&self) -> <Self::Register as Register>::Repr;
}
pub trait UnsafeWrite: Accessor {
    /// # Safety
    /// Writing this register has hardware-specific safety requirements which the caller must comply
    /// with.
    unsafe fn write(&self, value: <Self::Register as Register>::Repr);
}

pub trait Modify: Accessor + Read + Write {}

#[cfg(feature = "mmio-dynamic")]
pub mod dynamic_register {
    use crate::Register;
    use crate::access::{Access, Safe, Unsafe};
    use crate::accessor::{Accessor, Read, UnsafeRead, UnsafeWrite, Write};
    use core::marker::PhantomData;

    pub struct DynamicReg<Reg: Register, ReadAccess: Access, WriteAccess: Access>(
        *mut Reg::Repr,
        PhantomData<(ReadAccess, WriteAccess)>,
    );
    impl<Reg: Register, R: Access, W: Access> DynamicReg<Reg, R, W> {
        /// # Safety
        /// Creating concurrent instances of this type must be done with due consideration to the
        /// hardware's safety requirements.
        ///
        /// Additionally, the pointer must indeed point to a memory-mapped instance of this
        /// peripheral's register.
        pub unsafe fn summon_from_ptr(ptr: *mut Reg::Repr) -> Self {
            Self(ptr, PhantomData)
        }
    }
    impl<Reg: Register, R: Access, W: Access> Accessor for DynamicReg<Reg, R, W> {
        type Register = Reg;
    }
    impl<Reg: Register, W: Access> Read for DynamicReg<Reg, Safe, W> {
        fn read(&self) -> <Self::Register as Register>::Repr {
            unsafe { self.0.read_volatile() }
        }
    }
    impl<Reg: Register, R: Access> Write for DynamicReg<Reg, R, Safe> {
        fn write(&self, v: <Self::Register as Register>::Repr) {
            unsafe { self.0.write_volatile(v) }
        }
    }
    impl<Reg: Register, W: Access> UnsafeRead for DynamicReg<Reg, Unsafe, W> {
        unsafe fn read(&self) -> <Self::Register as Register>::Repr {
            unsafe { self.0.read_volatile() }
        }
    }
    impl<Reg: Register, R: Access> UnsafeWrite for DynamicReg<Reg, R, Unsafe> {
        unsafe fn write(&self, v: <Self::Register as Register>::Repr) {
            unsafe { self.0.write_volatile(v) }
        }
    }
}

#[cfg(feature = "mmio-static")]
pub mod static_register {
    use crate::Register;
    use crate::access::{Access, Safe, Unsafe};
    use crate::accessor::{Accessor, Read, UnsafeRead, UnsafeWrite, Write};
    use core::marker::PhantomData;

    pub struct StaticReg<
        Reg: Register,
        RA: Access,
        WA: Access,
        const BASE: usize,
        const OFFSET: usize
    >(
        PhantomData<(Reg, RA, WA)>,
    );
    impl<Reg: Register, R: Access, W: Access, const BASE: usize, const OFFSET: usize>
        StaticReg<Reg, R, W, BASE, OFFSET>
    {
        /// # Safety
        /// Creating concurrent instances of this type must be done with due consideration to the
        /// hardware's safety requirements.
        ///
        /// Additionally, `BASE+OFFSET` must be the address of an instance of this peripheral
        /// register in memory.
        pub const unsafe fn summon() -> Self {
            
            Self(PhantomData)
        }
        /// Produce a const pointer to the memory-mapped register.
        const fn as_ptr(&self) -> *const Reg::Repr {
            core::ptr::with_exposed_provenance::<Reg::Repr>(const { BASE.strict_add(OFFSET) })
        }
        /// Produce a mut pointer to the memory-mapped register.
        const fn as_mut_ptr(&self) -> *mut Reg::Repr {
            core::ptr::with_exposed_provenance_mut::<Reg::Repr>(const { BASE.strict_add(OFFSET) })
        }
    }
    impl<Reg: Register, R: Access, W: Access, const BASE: usize, const OFFSET: usize> Accessor
        for StaticReg<Reg, R, W, BASE, OFFSET>
    {
        type Register = Reg;
    }
    impl<Reg: Register, W: Access, const BASE: usize, const OFFSET: usize> Read
        for StaticReg<Reg, Safe, W, BASE, OFFSET>
    {
        fn read(&self) -> <Self::Register as Register>::Repr {
            unsafe { self.as_ptr().read_volatile() }
        }
    }
    impl<Reg: Register, W: Access, const BASE: usize, const OFFSET: usize> UnsafeRead
        for StaticReg<Reg, Unsafe, W, BASE, OFFSET>
    {
        unsafe fn read(&self) -> <Self::Register as Register>::Repr {
            unsafe { self.as_ptr().read_volatile() }
        }
    }
    impl<Reg: Register, R: Access, const BASE: usize, const OFFSET: usize> Write
        for StaticReg<Reg, R, Safe, BASE, OFFSET>
    {
        fn write(&self, value: <Self::Register as Register>::Repr) {
            unsafe { self.as_mut_ptr().write_volatile(value) }
        }
    }
    impl<Reg: Register, R: Access, const BASE: usize, const OFFSET: usize> UnsafeWrite
        for StaticReg<Reg, R, Unsafe, BASE, OFFSET>
    {
        unsafe fn write(&self, value: <Self::Register as Register>::Repr) {
            unsafe { self.as_mut_ptr().write_volatile(value) }
        }
    }
}
