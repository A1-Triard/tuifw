#[macro_export]
macro_rules! context {
    (struct $name:ident {
        $($field:ident : $ref_mut:tt $type_:ty),*
        $(,)?
    }) => {
        context! { @impl () $name { $($field : $ref_mut $type_),* } }
    };
    (pub ($($vis:tt)+) struct $name:ident {
        $($field:ident : $ref_mut:tt $type_:ty),*
        $(,)?
    }) => {
        context! { @impl (pub ($($vis)+)) $name { $($field : $ref_mut $type_),* } }
    };
    (pub struct $name:ident {
        $($field:ident : $ref_mut:tt $type_:ty),*
        $(,)?
    }) => {
        context! { @impl (pub) $name { $($field : $ref_mut $type_),* } }
    };
    (@impl ($($vis:tt)*) $name:ident {
        $($field:ident : $ref_mut:tt $type_:ty),*
    }) => {
        $($vis)* struct $name {
            $($field : context!(@impl * $ref_mut $type_)),*
        }
        impl $name {
            pub fn call<ContextCallReturnType>(
                $($field : context!(@impl & $ref_mut $type_)),*,
                f: impl std::ops::FnOnce(&mut Self) -> ContextCallReturnType 
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
    use std::mem::replace;

    context! {
        struct TestContext1 {
            a: const u8,
            b: ref u16,
            c: mut u32,
        }
    }

    #[test]
    fn test_context_1() {
        let mut x = 3;
        let res = TestContext1::call(1, &2, &mut x, |context| {
            assert_eq!(context.a(), 1u8);
            assert_eq!(context.b(), &2u16);
            assert_eq!(replace(context.c(), 12), 3u32);
            "res"
        });
        assert_eq!(res, "res");
        assert_eq!(x, 12);
    }
}
