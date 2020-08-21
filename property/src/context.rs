use std::marker::PhantomData;
use std::num::NonZeroUsize;
use std::ops::IndexMut;

#[tylift]
pub enum ContextFieldMutability {
    ContextFieldMut,
    ContextFieldRef
}

pub unsafe trait ContextToken {
    type Array: IndexMut<usize, Output=Option<NonZeroUsize>>;

    fn new_array() -> Self::Array;
}

pub unsafe trait ContextFieldIndex {
    type Token: ContextToken;
    const INDEX: usize;
}

pub trait ContextField: ContextFieldIndex {
    type Type;
    type Mutability: ContextFieldMutability;
}

pub struct Context<Token: ContextToken>(Token::Array, PhantomData<Token>);

impl<Token: ContextToken> Context<Token> {
    pub fn with_mut<
        T,
        Field1: ContextField<Token=Token, Mutability=ContextFieldMut>,
    >(
        v1: &mut Field1::Type,
        f: impl FnOnce(&mut Self) -> T
    ) -> T {
        let mut context = Context(Token::new_array(), PhantomData);
        let array: &mut Token::Array = &mut context.0;
        array[Field1::INDEX] = Some(unsafe { NonZeroUsize::new_unchecked(v1 as *mut _ as usize) });
        f(&mut context)
    }

    pub fn with_mut_mut<
        T,
        Field1: ContextField<Token=Token, Mutability=ContextFieldMut>,
        Field2: ContextField<Token=Token, Mutability=ContextFieldMut>,
    >(
        v1: &mut Field1::Type,
        v2: &mut Field2::Type,
        f: impl FnOnce(&mut Self) -> T
    ) -> T {
        let mut context = Context(Token::new_array(), PhantomData);
        let array: &mut Token::Array = &mut context.0;
        array[Field1::INDEX] = Some(unsafe { NonZeroUsize::new_unchecked(v1 as *mut _ as usize) });
        array[Field2::INDEX] = Some(unsafe { NonZeroUsize::new_unchecked(v2 as *mut _ as usize) });
        f(&mut context)
    }

    pub fn with_mut_ref<
        T,
        Field1: ContextField<Token=Token, Mutability=ContextFieldMut>,
        Field2: ContextField<Token=Token, Mutability=ContextFieldRef>,
    >(
        v1: &mut Field1::Type,
        v2: &Field2::Type,
        f: impl FnOnce(&mut Self) -> T
    ) -> T {
        let mut context = Context(Token::new_array(), PhantomData);
        let array: &mut Token::Array = &mut context.0;
        array[Field1::INDEX] = Some(unsafe { NonZeroUsize::new_unchecked(v1 as *mut _ as usize) });
        array[Field2::INDEX] = Some(unsafe { NonZeroUsize::new_unchecked(v2 as *const _ as usize) });
        f(&mut context)
    }

    pub fn with_ref_mut<
        T,
        Field1: ContextField<Token=Token, Mutability=ContextFieldRef>,
        Field2: ContextField<Token=Token, Mutability=ContextFieldMut>,
    >(
        v1: &Field1::Type,
        v2: &mut Field2::Type,
        f: impl FnOnce(&mut Self) -> T
    ) -> T {
        let mut context = Context(Token::new_array(), PhantomData);
        let array: &mut Token::Array = &mut context.0;
        array[Field1::INDEX] = Some(unsafe { NonZeroUsize::new_unchecked(v1 as *const _ as usize) });
        array[Field2::INDEX] = Some(unsafe { NonZeroUsize::new_unchecked(v2 as *mut _ as usize) });
        f(&mut context)
    }

    pub fn get_mut<Field: ContextField<Token=Token, Mutability=ContextFieldMut>>(&mut self) -> Option<&mut Field::Type> {
        let ptr = self.0[Field::INDEX].as_mut().map(|x| x.get() as *mut Field::Type);
        ptr.map(|ptr| unsafe { &mut *ptr })
    }

    pub fn get<Field: ContextField<Token=Token>>(&self) -> Option<&Field::Type> {
        let ptr = self.0[Field::INDEX].as_ref().map(|x| x.get() as *const Field::Type);
        ptr.map(|ptr| unsafe { &*ptr })
    }
}

#[macro_export]
macro_rules! context {
    ($token:ident; $(v:ident),*) => {
        context!(@impl [0usize] $token; $(v),*);
    };
    (@impl [$n:expr] $token:ident; $vn:ident $(, $vt:ident)*) => {
        unsafe impl $crate::context::ContextFieldIndex for $vn {
            type Token = $token;
            const INDEX: usize = $n;
        }
        context!(@impl [$n + 1] $token; $(vt),*);
    };
    (@impl [$n:expr] $token:ident; ) => {
        impl $crate::context::ContextToken for $token {
            type Array = [Option<NonZeroUsize>; $n];
        }
    };
}
