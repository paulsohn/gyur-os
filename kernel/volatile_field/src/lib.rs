#![no_std]

pub trait Structural<'a, T, A> {
    /// The concrete type of the return value.
    type StructuralType;

    /// Returns structural
    fn fields(self) -> Self::StructuralType;
}

pub use volatile_field_macros::StructuralOf;

// use volatile::{VolatilePtr, access::Access};
// use volatile::map_field;

// struct Foo {
//     x: u32,
//     y: i32,
// }

// pub struct StructuralOfFoo<'a, A: Access>(VolatilePtr<'a, Foo, A>);
// impl<'a, A: Access> StructuralOfFoo<'a, A> {
//     pub fn x(self) -> VolatilePtr<'a, u32, A> {
//         let ptr = self.0;
//         map_field!(ptr.x)
//     }
//     pub fn y(self) -> VolatilePtr<'a, i32, A> {
//         let ptr = self.0;
//         map_field!(ptr.y)
//     }
// }

// pub trait Structural<'a, T, A: Access> {
//     /// The concrete type of the return value.
//     type StructuralType;

//     /// Returns structural
//     fn fields(self) -> Self::StructuralType;
// }

// impl<'a, A: Access> Structural<'a, Foo, A> for VolatilePtr<'a, Foo, A> {
//     type StructuralType = StructuralOfFoo<'a, A>;

//     fn fields(self) -> Self::StructuralType {
//         StructuralOfFoo(self)
//     }
// }

// fn bar(v: VolatilePtr<'_, Foo, volatile::access::ReadWrite>) {
//     let a = v.fields().x();
//     let b = v.fields().y();
// }