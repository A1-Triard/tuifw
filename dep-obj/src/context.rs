#[doc(hidden)]
pub use core::ops::FnOnce as std_ops_FnOnce;

#[macro_export]
macro_rules! context {
    (mod $(< $( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? ),+ $(,)?>)? $name:ident  {
        $($field:ident $(($field_mut:ident))? : $ref_mut:tt $type_:ty ),*
        $(,)?
    }) => {
        mod $name {
            #[allow(unused_imports)]
            use super::*;

            context! { @impl Context [ $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?] [ $(< $( $lt ),+ >)?]
                {} {} {} {} { $($field $(($field_mut))? : $ref_mut $type_),* } }
        }
    };
    (@impl $c:ident [$($i:tt)*] [$($r:tt)*]
        {$({$($f:tt)*})*} {$({$($p:tt)*})*} {$({$($a:tt)*})*} {$({$($b:tt)*})*} {}) => {
        
        pub struct $c $($i)* {
            $($($f)*),*
        }

        impl $($i)* $c $($r)* {
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

        unsafe impl $($i)* Send for $c $($r)* { }
        unsafe impl $($i)* Sync for $c $($r)* { }
    };
    (@impl $c:ident [$($i:tt)*] [$($r:tt)*]
        {$({$($f:tt)*})*} {$({$($p:tt)*})*} {$({$($a:tt)*})*} {$({$($b:tt)*})*}
        {$field:ident : ref $type_:ty $(, $ft:ident $(($fm:ident))? : $rt:tt $t:ty)*}) => {

        context! { @impl $c [$($i)*] [$($r)*]
            {$({$($f)*})* {$field : *const $type_}}
            {$({$($p)*})* {$field : &$type_}}
            {$({$($a)*})* {$field : $field as *const $type_}}
            {$({$($b)*})* {
                pub fn $field (&self) -> &$type_ { unsafe { &*self.$field } }
            }}
            {$($ft $(($fm))? : $rt $t),*}
        }
    };
    (@impl $c:ident [$($i:tt)*] [$($r:tt)*]
        {$({$($f:tt)*})*} {$({$($p:tt)*})*} {$({$($a:tt)*})*} {$({$($b:tt)*})*}
        {$field:ident ($field_mut:ident) : mut $type_:ty $(, $ft:ident $(($fm:ident))? : $rt:tt $t:ty)*}) => {

        context! { @impl $c [$($i)*] [$($r)*]
            {$({$($f)*})* {$field : *mut $type_}}
            {$({$($p)*})* {$field : &mut $type_}}
            {$({$($a)*})* {$field : $field as *mut $type_}}
            {$({$($b)*})* {
                pub fn $field (&self) -> &$type_ { unsafe { &*self.$field } }
                pub fn $field_mut (&mut self) -> &mut $type_ { unsafe { &mut *self.$field } }
            }}
            {$($ft $(($fm))? : $rt $t),*}
        }
    };
    (@impl $c:ident [$($i:tt)*] [$($r:tt)*]
        {$({$($f:tt)*})*} {$({$($p:tt)*})*} {$({$($a:tt)*})*} {$({$($b:tt)*})*}
        {$field:ident : const $type_:ty $(, $ft:ident $(($fm:ident))? : $rt:tt $t:ty)*}) => {

        context! { @impl $c [$($i)*] [$($r)*]
            {$({$($f)*})* {$field : $type_}}
            {$({$($p)*})* {$field : $type_}}
            {$({$($a)*})* {$field}}
            {$({$($b)*})* {
                pub fn $field (&self) -> $type_ { self.$field }
            }}
            {$($ft $(($fm))? : $rt $t),*}
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
