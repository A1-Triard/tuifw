pub struct ContextMut<T1>(*mut T1);

pub struct ContextRef<T1>(*const T1);

pub struct ContextMutMut<T1, T2>(*mut T1, *mut T2);

pub struct ContextMutRef<T1, T2>(*mut T1, *const T2);

pub struct ContextRefRef<T1, T2>(*const T1, *const T2);

pub struct Context(());

impl Context {
    pub fn with_mut<
        T,
        T1,
    >(
        v1: &mut T1,
        f: impl FnOnce(&mut ContextMut<T1>) -> T
    ) -> T {
        let mut context = ContextMut(
            v1 as *mut _,
        );
        f(&mut context)
    }

    pub fn with_ref<
        T,
        T1,
    >(
        v1: &T1,
        f: impl FnOnce(&mut ContextRef<T1>) -> T
    ) -> T {
        let mut context = ContextRef(
            v1 as *const _,
        );
        f(&mut context)
    }

    pub fn with_mut_mut<
        T,
        T1,
        T2,
    >(
        v1: &mut T1,
        v2: &mut T2,
        f: impl FnOnce(&mut ContextMutMut<T1, T2>) -> T
    ) -> T {
        let mut context = ContextMutMut(
            v1 as *mut _,
            v2 as *mut _,
        );
        f(&mut context)
    }

    pub fn with_mut_ref<
        T,
        T1,
        T2,
    >(
        v1: &mut T1,
        v2: &T2,
        f: impl FnOnce(&mut ContextMutRef<T1, T2>) -> T
    ) -> T {
        let mut context = ContextMutRef(
            v1 as *mut _,
            v2 as *const _,
        );
        f(&mut context)
    }

    pub fn with_ref_ref<
        T,
        T1,
        T2,
    >(
        v1: &T1,
        v2: &T2,
        f: impl FnOnce(&mut ContextRefRef<T1, T2>) -> T
    ) -> T {
        let mut context = ContextRefRef(
            v1 as *const _,
            v2 as *const _,
        );
        f(&mut context)
    }
}

impl<T1> ContextMut<T1> {
    pub fn get_1(&mut self) -> &mut T1 { unsafe { &mut *self.0 } }
}

impl<T1> ContextRef<T1> {
    pub fn get_1(&self) -> &T1 { unsafe { &*self.0 } }
}

impl<T1, T2> ContextMutMut<T1, T2> {
    pub fn get_1(&mut self) -> &mut T1 { unsafe { &mut *self.0 } }

    pub fn get_2(&mut self) -> &mut T2 { unsafe { &mut *self.1 } }
}

impl<T1, T2> ContextMutRef<T1, T2> {
    pub fn get_1(&mut self) -> &mut T1 { unsafe { &mut *self.0 } }

    pub fn get_2(&self) -> &T2 { unsafe { &*self.1 } }
}

impl<T1, T2> ContextRefRef<T1, T2> {
    pub fn get_1(&self) -> &T1 { unsafe { &*self.0 } }

    pub fn get_2(&self) -> &T2 { unsafe { &*self.1 } }
}
