#[macro_export]
macro_rules! context {
    (
        mod $name:ident
        $(< $( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? ),+ $(,)?>)?
        {
            $($(
                $field:ident $(/ $field_mut:ident)? : $field_mod:ident $field_ty:ty
            ),+ $(,)?)?
        }
    ) => {
        mod $name {
            #[allow(unused_imports)]
            use super::*;

            context! {
                @impl Context
                [ $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?] [ $(< $( $lt ),+ >)?]
                {} {} {} {} { $($($field $(/ $field_mut)? : $field_mod $field_ty),+)? }
            }
        }
    };
    (
        @impl $name:ident
        [$($g:tt)*] [$($r:tt)*]
        {$({$($f:tt)*})*}
        {$({$($p:tt)*})*}
        {$({$($a:tt)*})*}
        {$({$($b:tt)*})*}
        {$field:ident : ref $ty:ty $(, $($other_fields:tt)+)?}
    ) => {
        context! {
            @impl $name
            [$($g)*] [$($r)*]
            {
                $({$($f)*})*
                {$field : *const $ty}
            }
            {
                $({$($p)*})*
                {$field : &$ty}
            }
            {
                $({$($a)*})*
                {$field : $field as *const $ty}
            }
            {
                $({$($b)*})*
                {
                    pub fn $field (&self) -> &$ty { unsafe { &*self.$field } }
                }
            }
            {$($($other_fields)+)?}
        }
    };
    (
        @impl $name:ident
        [$($g:tt)*] [$($r:tt)*]
        {$({$($f:tt)*})*}
        {$({$($p:tt)*})*}
        {$({$($a:tt)*})*}
        {$({$($b:tt)*})*}
        {$field:ident / $field_mut:ident : mut $ty:ty $(, $($other_fields:tt)+)?}
    ) => {
        context! {
            @impl $name
            [$($g)*] [$($r)*]
            {
                $({$($f)*})*
                {$field : *mut $ty}
            }
            {
                $({$($p)*})*
                {$field : &mut $ty}
            }
            {
                $({$($a)*})*
                {$field : $field as *mut $ty}
            }
            {
                $({$($b)*})*
                {
                    #[allow(dead_code)]
                    pub fn $field (&self) -> &$ty { unsafe { &*self.$field } }
                    #[allow(dead_code)]
                    pub fn $field_mut (&mut self) -> &mut $ty { unsafe { &mut *self.$field } }
                }
            }
            {$($($other_fields)+)?}
        }
    };
    (
        @impl $name:ident
        [$($g:tt)*] [$($r:tt)*]
        {$({$($f:tt)*})*}
        {$({$($p:tt)*})*}
        {$({$($a:tt)*})*}
        {$({$($b:tt)*})*}
        {$field:ident : const $ty:ty $(, $($other_fields:tt)+)?}
    ) => {
        context! {
            @impl $name
            [$($g)*] [$($r)*]
            {
                $({$($f)*})*
                {$field : $ty}
            }
            {
                $({$($p)*})*
                {$field : $ty}
            }
            {
                $({$($a)*})*
                {$field}
            }
            {
                $({$($b)*})*
                {
                    pub fn $field (&self) -> $ty { self.$field }
                }
            }
            {$($($other_fields)+)?}
        }
    };
    (
        @impl $name:ident
        [$($g:tt)*] [$($r:tt)*]
        {$({$($f:tt)*})*} {$({$($p:tt)*})*} {$({$($a:tt)*})*} {$({$($b:tt)*})*} {}
    ) => {
        pub struct $name $($g)* {
            $($($f)*),*
        }

        impl $($g)* $name $($r)* {
            pub fn call<ContextCallReturnType>(
                $($($p)*),*,
                f: impl $crate::std_ops_FnOnce(&mut Self) -> ContextCallReturnType 
            ) -> ContextCallReturnType {
                let mut context = Self {
                    $($($a)*),*
                };
                f(&mut context)
            }

            $($($b)*)*
        }

        unsafe impl $($g)* Send for $name $($r)* { }
        unsafe impl $($g)* Sync for $name $($r)* { }
    };
}

#[cfg(docsrs)]
pub mod example {
    use core::fmt::Display;

    pub struct Data {
        pub x: i16,
        pub y: i16
    }

    context! {
        mod example_context {
            data/data_mut: mut Data,
            display: ref dyn Display,
            id: const usize,
        }
    }

    pub use example_context::Context as ExampleContext;
}

#[cfg(test)]
mod test {
    use core::mem::replace;

    context! {
        mod context_1 {
            a: const u8,
            b: ref u16,
            c/c_mut: mut u32,
        }
    }

    type Context1 = context_1::Context;

    #[test]
    fn test_context_1() {
        let mut x = 3;
        let res = Context1::call(1, &2, &mut x, |context| {
            assert_eq!(context.a(), 1u8);
            assert_eq!(context.b(), &2u16);
            assert_eq!(replace(context.c_mut(), 12), 3u32);
            "res"
        });
        assert_eq!(res, "res");
        assert_eq!(x, 12);
    }

    context! {
        mod context_2 {
            a: const u8,
            b: ref u16,
            c/c_mut: mut u32,
        }
    }

    pub type Context2 = context_2::Context;

    #[test]
    fn test_context_2() {
        let mut x = 3;
        let res = Context2::call(1, &2, &mut x, |context| {
            assert_eq!(context.a(), 1u8);
            assert_eq!(context.b(), &2u16);
            assert_eq!(replace(context.c_mut(), 12), 3u32);
            "res"
        });
        assert_eq!(res, "res");
        assert_eq!(x, 12);
    }
}
