#[doc(hidden)]
pub use core::ops::FnOnce as std_ops_FnOnce;

#[macro_export]
macro_rules! context {
    (mod $name:ident {
        $($field:ident : $ref_mut:tt $type_:ty),*
        $(,)?
    }) => {
        mod $name {
            #[allow(unused_imports)]
            use super::*;

            pub struct Context {
                $($field : context!(@impl * $ref_mut $type_)),*
            }

            impl Context {
                pub fn call<ContextCallReturnType>(
                    $($field : context!(@impl & $ref_mut $type_)),*,
                    f: impl $crate::context::std_ops_FnOnce(&mut Self) -> ContextCallReturnType 
                ) -> ContextCallReturnType {
                    let mut context = Self {
                        $($field : context!(@impl as $field $ref_mut $type_)),*
                    };
                    f(&mut context)
                }
                $(
                    context! { @impl fn $field $ref_mut $type_ }
                )*
            }

            unsafe impl Send for Context { }
            unsafe impl Sync for Context { }
        }
    };
    (@impl * ref $type_:ty) => { *const $type_ };
    (@impl * mut $type_:ty) => { *mut $type_ };
    (@impl * const $type_:ty) => { $type_ };
    (@impl & ref $type_:ty) => { &$type_ };
    (@impl & mut $type_:ty) => { &mut $type_ };
    (@impl & const $type_:ty) => { $type_ };
    (@impl as $field:ident ref $type_:ty) => { $field as *const $type_ };
    (@impl as $field:ident mut $type_:ty) => { $field as *mut $type_ };
    (@impl as $field:ident const $type_:ty) => { $field };
    (@impl fn $field:ident ref $type_:ty) => {
        pub fn $field (&self) -> &$type_ { unsafe { &*self.$field } }
    };
    (@impl fn $field:ident mut $type_:ty) => {
        pub fn $field (&mut self) -> &mut $type_ { unsafe { &mut *self.$field } }
    };
    (@impl fn $field:ident const $type_:ty) => {
        pub fn $field (&self) -> $type_ { self.$field }
    };
}

#[cfg(test)]
mod test {
    use core::mem::replace;

    context! {
        mod context_1 {
            a: const u8,
            b: ref u16,
            c: mut u32,
        }
    }

    type Context1 = context_1::Context;

    #[test]
    fn test_context_1() {
        let mut x = 3;
        let res = Context1::call(1, &2, &mut x, |context| {
            assert_eq!(context.a(), 1u8);
            assert_eq!(context.b(), &2u16);
            assert_eq!(replace(context.c(), 12), 3u32);
            "res"
        });
        assert_eq!(res, "res");
        assert_eq!(x, 12);
    }

    context! {
        mod context_2 {
            a: const u8,
            b: ref u16,
            c: mut u32,
        }
    }

    pub type Context2 = context_2::Context;

    #[test]
    fn test_context_2() {
        let mut x = 3;
        let res = Context2::call(1, &2, &mut x, |context| {
            assert_eq!(context.a(), 1u8);
            assert_eq!(context.b(), &2u16);
            assert_eq!(replace(context.c(), 12), 3u32);
            "res"
        });
        assert_eq!(res, "res");
        assert_eq!(x, 12);
    }
}
