#![feature(effects)]

#![deny(warnings)]
#![doc(test(attr(deny(warnings))))]
#![doc(test(attr(allow(dead_code))))]
#![doc(test(attr(allow(unused_variables))))]
#![allow(clippy::blocks_in_if_conditions)]
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::nonminimal_bool)]
#![allow(clippy::type_complexity)]

#![no_std]

extern crate alloc;

mod stack_panel;
pub use stack_panel::*;

mod dock_panel;
pub use dock_panel::*;

mod static_text;
pub use static_text::*;

mod background;
pub use background::*;

mod input_line;
pub use input_line::*;

mod button;
pub use button::*;

mod frame;
pub use frame::*;

#[doc(hidden)]
pub use alloc::boxed::Box as alloc_boxed_Box;
#[doc(hidden)]
pub use alloc::borrow::Cow as alloc_borrow_Cow;
#[doc(hidden)]
pub use alloc::string::String as alloc_string_String;
#[doc(hidden)]
pub use core::mem::replace as core_mem_replace;
#[doc(hidden)]
pub use paste::paste as paste_paste;
#[doc(hidden)]
pub use timer_no_std::MonoClock as timer_no_std_MonoClock;
#[doc(hidden)]
pub use tuifw_screen_base::Error as tuifw_screen_base_Error;
#[doc(hidden)]
pub use tuifw_screen_base::Screen as tuifw_screen_base_Screen;
#[doc(hidden)]
pub use tuifw_window::Window as tuifw_window_Window;
#[doc(hidden)]
pub use tuifw_window::WindowTree as tuifw_window_WindowTree;

#[macro_export]
macro_rules! widget {
    (
        $W:ident
    ) => {
        pub fn window<State: ?Sized>(
            self,
            tree: &mut $crate::tuifw_window_WindowTree<State>,
            parent: $crate::tuifw_window_Window<State>,
            prev: Option<$crate::tuifw_window_Window<State>>
        ) -> Result<$crate::tuifw_window_Window<State>, $crate::tuifw_screen_base_Error> {
            $crate::tuifw_window_Window::new(
                tree,
                $crate::alloc_boxed_Box::new($W),
                $crate::alloc_boxed_Box::new(self),
                parent,
                prev
            )
        }

        pub fn window_tree<State: ?Sized>(
            self,
            screen: $crate::alloc_boxed_Box<dyn $crate::tuifw_screen_base_Screen>,
            clock: &$crate::timer_no_std_MonoClock,
        ) -> Result<$crate::tuifw_window_WindowTree<State>, $crate::tuifw_screen_base_Error> {
            $crate::tuifw_window_WindowTree::new(
                screen,
                clock,
                $crate::alloc_boxed_Box::new($W),
                $crate::alloc_boxed_Box::new(self)
            )
        }
    };
    (
        $W:ident; $init:ident
    ) => {
        pub fn window<State: ?Sized>(
            self,
            tree: &mut $crate::tuifw_window_WindowTree<State>,
            parent: $crate::tuifw_window_Window<State>,
            prev: Option<$crate::tuifw_window_Window<State>>
        ) -> Result<$crate::tuifw_window_Window<State>, $crate::tuifw_screen_base_Error> {
            let w = $crate::tuifw_window_Window::new(
                tree,
                $crate::alloc_boxed_Box::new($W),
                $crate::alloc_boxed_Box::new(self),
                parent,
                prev
            )?;
            Self::$init(tree, w);
            Ok(w)
        }

        pub fn window_tree<State: ?Sized>(
            self,
            screen: $crate::alloc_boxed_Box<dyn $crate::tuifw_screen_base_Screen>,
            clock: &$crate::timer_no_std_MonoClock,
        ) -> Result<$crate::tuifw_window_WindowTree<State>, $crate::tuifw_screen_base_Error> {
            let mut tree = $crate::tuifw_window_WindowTree::new(
                screen,
                clock,
                $crate::alloc_boxed_Box::new($W),
                $crate::alloc_boxed_Box::new(self)
            )?;
            let root = tree.root();
            Self::$init(&mut tree, root);
            Ok(tree)
        }
    };
}

#[macro_export]
macro_rules! prop_value {
    (
        $name:ident : $ty:ty $(; $on_changed:ident)? $(| $assert:ident)?
    ) => {
        $crate::paste_paste! {
            pub fn $name<State: ?Sized>(
                tree: &$crate::tuifw_window_WindowTree<State>,
                window: $crate::tuifw_window_Window<State>
            ) -> $ty {
                window.data::<Self>(tree).$name
            }

            pub fn [< set_ $name >] <State: ?Sized>(
                tree: &mut $crate::tuifw_window_WindowTree<State>,
                window: $crate::tuifw_window_Window<State>,
                value: $ty
            ) {
                let data = window.data_mut::<Self>(tree);
                $(data.$assert(value);)?
                data.$name = value;
                $(Self::$on_changed(tree, window);)?
            }
        }
    };
}

#[macro_export]
macro_rules! prop_value_measure {
    (
        $name:ident : $ty:ty $(; $on_changed:ident)? $(| $assert:ident)?
    ) => {
        $crate::paste_paste! {
            pub fn $name<State: ?Sized>(
                tree: &$crate::tuifw_window_WindowTree<State>,
                window: $crate::tuifw_window_Window<State>
            ) -> $ty {
                window.data::<Self>(tree).$name
            }

            pub fn [< set_ $name >] <State: ?Sized>(
                tree: &mut $crate::tuifw_window_WindowTree<State>,
                window: $crate::tuifw_window_Window<State>,
                value: $ty
            ) {
                let data = window.data_mut::<Self>(tree);
                $(data.$assert(value);)?
                data.$name = value;
                $(Self::$on_changed(tree, window);)?
            }
        }
    };
}

#[macro_export]
macro_rules! prop_value_render {
    (
        $name:ident : $ty:ty $(; $on_changed:ident)? $(| $assert:ident)?
    ) => {
        $crate::paste_paste! {
            pub fn $name<State: ?Sized>(
                tree: &$crate::tuifw_window_WindowTree<State>,
                window: $crate::tuifw_window_Window<State>
            ) -> $ty {
                window.data::<Self>(tree).$name
            }

            pub fn [< set_ $name >] <State: ?Sized>(
                tree: &mut $crate::tuifw_window_WindowTree<State>,
                window: $crate::tuifw_window_Window<State>,
                value: $ty
            ) {
                let data = window.data_mut::<Self>(tree);
                $(data.$assert(value);)?
                data.$name = value;
                window.invalidate_render(tree);
                $(Self::$on_changed(tree, window);)?
            }
        }
    };
}

#[macro_export]
macro_rules! prop_string {
    (
        $name:ident $(; $on_changed:ident)?
    ) => {
        $crate::paste_paste! {
            pub fn $name<'a, State: ?Sized>(
                tree: &'a $crate::tuifw_window_WindowTree<State>,
                window: $crate::tuifw_window_Window<State>
            ) -> &'a $crate::alloc_string_String {
                &window.data::<Self>(tree).$name
            }

            pub fn [< $name _mut >] <State: ?Sized, T>(
                tree: &mut $crate::tuifw_window_WindowTree<State>,
                window: $crate::tuifw_window_Window<State>,
                f: impl FnOnce(&mut $crate::alloc_string_String) -> T
            ) -> T {
                let value = &mut window.data_mut::<Self>(tree).$name;
                let res = f(value);
                $(Self::$on_changed(tree, window);)?
                res
            }

            pub fn [< set_ $name >] <'a, State: ?Sized>(
                tree: &mut $crate::tuifw_window_WindowTree<State>,
                window: $crate::tuifw_window_Window<State>,
                value: impl Into<$crate::alloc_borrow_Cow<'a, str>>
            ) {
                Self:: [< $name _mut >] (
                    tree,
                    window,
                    |x| $crate::core_mem_replace(x, value.into().into_owned())
                );
            }
        }
    };
}

#[macro_export]
macro_rules! prop_string_measure {
    (
        $name:ident $(; $on_changed:ident)?
    ) => {
        $crate::paste_paste! {
            pub fn $name<'a, State: ?Sized>(
                tree: &'a $crate::tuifw_window_WindowTree<State>,
                window: $crate::tuifw_window_Window<State>
            ) -> &'a $crate::alloc_string_String {
                &window.data::<Self>(tree).$name
            }

            pub fn [< $name _mut >] <State: ?Sized, T>(
                tree: &mut $crate::tuifw_window_WindowTree<State>,
                window: $crate::tuifw_window_Window<State>,
                f: impl FnOnce(&mut $crate::alloc_string_String) -> T
            ) -> T {
                let value = &mut window.data_mut::<Self>(tree).$name;
                let res = f(value);
                window.invalidate_measure(tree);
                $(Self::$on_changed(tree, window);)?
                res
            }

            pub fn [< set_ $name >] <'a, State: ?Sized>(
                tree: &mut $crate::tuifw_window_WindowTree<State>,
                window: $crate::tuifw_window_Window<State>,
                value: impl Into<$crate::alloc_borrow_Cow<'a, str>>
            ) {
                Self:: [< $name _mut >] (
                    tree,
                    window,
                    |x| $crate::core_mem_replace(x, value.into().into_owned())
                );
            }
        }
    };
}

#[macro_export]
macro_rules! prop_string_render {
    (
        $name:ident $(; $on_changed:ident)?
    ) => {
        $crate::paste_paste! {
            pub fn $name<'a, State: ?Sized>(
                tree: &'a $crate::tuifw_window_WindowTree<State>,
                window: $crate::tuifw_window_Window<State>
            ) -> &'a $crate::alloc_string_String {
                &window.data::<Self>(tree).$name
            }

            pub fn [< $name _mut >] <State: ?Sized, T>(
                tree: &mut $crate::tuifw_window_WindowTree<State>,
                window: $crate::tuifw_window_Window<State>,
                f: impl FnOnce(&mut $crate::alloc_string_String) -> T
            ) -> T {
                let value = &mut window.data_mut::<Self>(tree).$name;
                let res = f(value);
                window.invalidate_render(tree);
                $(Self::$on_changed(tree, window);)?
                res
            }

            pub fn [< set_ $name >] <'a, State: ?Sized>(
                tree: &mut $crate::tuifw_window_WindowTree<State>,
                window: $crate::tuifw_window_Window<State>,
                value: impl Into<$crate::alloc_borrow_Cow<'a, str>>
            ) {
                Self:: [< $name _mut >] (
                    tree,
                    window,
                    |x| $crate::core_mem_replace(x, value.into().into_owned())
                );
            }
        }
    };
}

#[macro_export]
macro_rules! prop_obj_render {
    (
        $name:ident : $ty:ty $(; $on_changed:ident)?
    ) => {
        $crate::paste_paste! {
            pub fn $name<'a, State: ?Sized>(
                tree: &'a $crate::tuifw_window_WindowTree<State>,
                window: $crate::tuifw_window_Window<State>
            ) -> &'a $ty {
                &window.data::<Self>(tree).$name
            }

            pub fn [< $name _mut >] <State: ?Sized, T>(
                tree: &mut $crate::tuifw_window_WindowTree<State>,
                window: $crate::tuifw_window_Window<State>,
                f: impl FnOnce(&mut $ty) -> T
            ) -> T {
                let value = &mut window.data_mut::<Self>(tree).$name;
                let res = f(value);
                window.invalidate_render(tree);
                $(Self::$on_changed(tree, window);)?
                res
            }

            pub fn [< set_ $name >] <State: ?Sized>(
                tree: &mut $crate::tuifw_window_WindowTree<State>,
                window: $crate::tuifw_window_Window<State>,
                value: $ty
            ) {
                Self:: [< $name _mut >] (
                    tree,
                    window,
                    |x| $crate::core_mem_replace(x, value)
                );
            }
        }
    };
}
