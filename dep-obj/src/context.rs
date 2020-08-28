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

            context! { @impl Context {} {} {} {} { $($field : $ref_mut $type_),* } }
        }
    };
    (@impl $c:ident {$({$($f:tt)*})*} {$({$($p:tt)*})*} {$({$($a:tt)*})*} {$({$($b:tt)*})*} {}) => {
        pub struct $c {
            $($($f)*),*
        }

        impl $c {
            pub fn call<ContextCallReturnType>(
                $($($p)*),*,
                f: impl $crate::context::std_ops_FnOnce(&mut Self) -> ContextCallReturnType 
            ) -> ContextCallReturnType {
                let mut context = Self {
                    $($($a)*),*
                };
                f(&mut context)
            }

            $($($b)*)*
        }

        unsafe impl Send for $c { }
        unsafe impl Sync for $c { }
    };
    (@impl $c:ident {$({$($f:tt)*})*} {$({$($p:tt)*})*} {$({$($a:tt)*})*} {$({$($b:tt)*})*}
        {$field:ident : ref $type_:ty $(, $ft:ident : $rt:tt $t:ty)*}) => {

        context! { @impl $c
            {$({$($f)*})* {$field : *const $type_}}
            {$({$($p)*})* {$field : &$type_}}
            {$({$($a)*})* {$field : $field as *const $type_}}
            {$({$($b)*})* {
                pub fn $field (&self) -> &$type_ { unsafe { &*self.$field } }
            }}
            {$($ft : $rt $t),*}
        }
    };
    (@impl $c:ident {$({$($f:tt)*})*} {$({$($p:tt)*})*} {$({$($a:tt)*})*} {$({$($b:tt)*})*}
        {$field:ident : mut $type_:ty $(, $ft:ident : $rt:tt $t:ty)*}) => {

        context! { @impl $c
            {$({$($f)*})* {$field : *mut $type_}}
            {$({$($p)*})* {$field : &mut $type_}}
            {$({$($a)*})* {$field : $field as *mut $type_}}
            {$({$($b)*})* {
                pub fn $field (&mut self) -> &mut $type_ { unsafe { &mut *self.$field } }
            }}
            {$($ft : $rt $t),*}
        }
    };
    (@impl $c:ident {$({$($f:tt)*})*} {$({$($p:tt)*})*} {$({$($a:tt)*})*} {$({$($b:tt)*})*}
        {$field:ident : const $type_:ty $(, $ft:ident : $rt:tt $t:ty)*}) => {

        context! { @impl $c
            {$({$($f)*})* {$field : $type_}}
            {$({$($p)*})* {$field : $type_}}
            {$({$($a)*})* {$field}}
            {$({$($b)*})* {
                pub fn $field (&self) -> $type_ { self.$field }
            }}
            {$($ft : $rt $t),*}
        }
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
