mod private {
    pub trait Sealed {}
}

pub trait Access : private::Sealed {}

pub struct Safe(!);
impl Access for Safe {}
impl private::Sealed for Safe {}

pub struct Unsafe(!);
impl Access for Unsafe {}
impl private::Sealed for Unsafe {}

pub struct NoAccess(!);
impl Access for NoAccess {}
impl private::Sealed for NoAccess {}

