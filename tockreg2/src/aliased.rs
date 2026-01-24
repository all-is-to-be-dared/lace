use core::marker::PhantomData;
use crate::{Register, UIntLike};

pub struct Aliased<Read: Register, Write: Register> {
    /// This type is uninhabitable
    _force_uninhabited: !,
    _phantom: PhantomData<(Read, Write)>,
}
impl<Repr: UIntLike, Read, Write> Register
for Aliased<Read, Write>
where
    Read: Register<Repr = Repr>,
    Write: Register<Repr = Repr>,
{
    type Repr = Repr;
    type ReadIdentity = Read::ReadIdentity;
    type WriteIdentity = Write::WriteIdentity;
}
