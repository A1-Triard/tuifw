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

mod label;
pub use label::*;

mod check_box;
pub use check_box::*;

mod radio_button;
pub use radio_button::*;

#[doc(hidden)]
pub use alloc::boxed::Box as alloc_boxed_Box;
#[doc(hidden)]
pub use alloc::borrow::Cow as alloc_borrow_Cow;
#[doc(hidden)]
pub use alloc::string::String as alloc_string_String;
#[doc(hidden)]
pub use core::compile_error as core_compile_error;
#[doc(hidden)]
pub use core::concat as core_concat;
#[doc(hidden)]
pub use core::mem::replace as core_mem_replace;
#[doc(hidden)]
pub use core::ops::Deref as core_ops_Deref;
#[doc(hidden)]
pub use core::stringify as core_stringify;
#[doc(hidden)]
pub use paste::paste as paste_paste;
#[doc(hidden)]
pub use timer_no_std::MonoClock as timer_no_std_MonoClock;
#[doc(hidden)]
pub use tuifw_screen_base::Error as tuifw_screen_base_Error;
#[doc(hidden)]
pub use tuifw_screen_base::Screen as tuifw_screen_base_Screen;
#[doc(hidden)]
pub use tuifw_window::App as tuifw_window_App;
#[doc(hidden)]
pub use tuifw_window::WidgetData as tuifw_window_WidgetData;
#[doc(hidden)]
pub use tuifw_window::Window as tuifw_window_Window;
#[doc(hidden)]
pub use tuifw_window::WindowTree as tuifw_window_WindowTree;

#[macro_export]
macro_rules! widget {
    (
        #[widget($Widget:ident $(, init=$init:ident)? $(, drop=$drop:ident)?)]
        $vis:vis struct $name:ident {
            $($(
                $(#[property$(($($attrs:tt)*))?])?
                $field_name:ident : $field_ty:ty
            ),+ $(,)?)?
        }
    ) => {
        $vis struct $name {
            $($($field_name: $field_ty),+)?
        }

        impl $crate::tuifw_window_WidgetData for $name {
            fn drop_widget_data(
                &mut self,
                #[allow(unused_variables)]
                tree: &mut $crate::tuifw_window_WindowTree,
                #[allow(unused_variables)]
                app: &mut dyn $crate::tuifw_window_App
            ) {
                $(self.$drop(tree, app);)?
            }
        }

        impl $name {
            $vis fn new(
                tree: &mut $crate::tuifw_window_WindowTree,
                parent: Option<$crate::tuifw_window_Window>,
                prev: Option<$crate::tuifw_window_Window>
            ) -> Result<$crate::tuifw_window_Window, $crate::tuifw_screen_base_Error> {
                let w = $crate::tuifw_window_Window::new(
                    tree,
                    $crate::alloc_boxed_Box::new($Widget),
                    parent,
                    prev
                )?;
                $(Self::$init(tree, w);)?
                Ok(w)
            }

            $($($crate::widget_impl! {
                $(#[property($($($attrs)*)?)])?
                $vis $field_name : $field_ty
            })+)?
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! widget_impl {
    (
        $vis:vis $field_name:ident : $field_ty:ty
    ) => {
    };
    (
        #[property(value, measure $(, changed=$on_changed:ident)?)]
        $vis:vis $name:ident : $ty:ty
    ) => {
        $crate::paste_paste! {
            $vis fn $name(
                tree: &$crate::tuifw_window_WindowTree,
                window: $crate::tuifw_window_Window
            ) -> $ty {
                window.data::<Self>(tree).$name
            }

            $vis fn [< set_ $name >] (
                tree: &mut $crate::tuifw_window_WindowTree,
                window: $crate::tuifw_window_Window,
                value: $ty
            ) {
                let data = window.data_mut::<Self>(tree);
                data.$name = value;
                window.invalidate_measure(tree);
                $(Self::$on_changed(tree, window);)?
            }
        }
    };
    (
        #[property(value, render $(, changed=$on_changed:ident)?)]
        $vis:vis $name:ident : $ty:ty
    ) => {
        $crate::paste_paste! {
            $vis fn $name(
                tree: &$crate::tuifw_window_WindowTree,
                window: $crate::tuifw_window_Window
            ) -> $ty {
                window.data::<Self>(tree).$name
            }

            $vis fn [< set_ $name >] (
                tree: &mut $crate::tuifw_window_WindowTree,
                window: $crate::tuifw_window_Window,
                value: $ty
            ) {
                let data = window.data_mut::<Self>(tree);
                data.$name = value;
                window.invalidate_render(tree);
                $(Self::$on_changed(tree, window);)?
            }
        }
    };
    (
        #[property(value $(, changed=$on_changed:ident)?)]
        $vis:vis $name:ident : $ty:ty
    ) => {
        $crate::paste_paste! {
            $vis fn $name(
                tree: &$crate::tuifw_window_WindowTree,
                window: $crate::tuifw_window_Window
            ) -> $ty {
                window.data::<Self>(tree).$name
            }

            $vis fn [< set_ $name >] (
                tree: &mut $crate::tuifw_window_WindowTree,
                window: $crate::tuifw_window_Window,
                value: $ty
            ) {
                let data = window.data_mut::<Self>(tree);
                data.$name = value;
                $(Self::$on_changed(tree, window);)?
            }
        }
    };
    (
        #[property(ref, measure $(, changed=$on_changed:ident)?)]
        $vis:vis $name:ident : $ty:ty
    ) => {
        $crate::paste_paste! {
            $vis fn $name<'a>(
                tree: &'a $crate::tuifw_window_WindowTree,
                window: $crate::tuifw_window_Window
            ) -> &'a $ty {
                &window.data::<Self>(tree).$name
            }

            $vis fn [< $name _mut >] <T>(
                tree: &mut $crate::tuifw_window_WindowTree,
                window: $crate::tuifw_window_Window,
                f: impl FnOnce(&mut $ty) -> T
            ) -> T {
                let value = &mut window.data_mut::<Self>(tree).$name;
                let res = f(value);
                window.invalidate_measure(tree);
                $(Self::$on_changed(tree, window);)?
                res
            }

            $vis fn [< set_ $name >] <'a>(
                tree: &mut $crate::tuifw_window_WindowTree,
                window: $crate::tuifw_window_Window,
                value: impl Into<$crate::alloc_borrow_Cow<'a, <$ty as $crate::core_ops_Deref>::Target>>
            ) {
                Self:: [< $name _mut >] (
                    tree,
                    window,
                    |x| $crate::core_mem_replace(x, value.into().into_owned())
                );
            }
        }
    };
    (
        #[property(ref, render $(, changed=$on_changed:ident)?)]
        $vis:vis $name:ident : $ty:ty
    ) => {
        $crate::paste_paste! {
            $vis fn $name<'a>(
                tree: &'a $crate::tuifw_window_WindowTree,
                window: $crate::tuifw_window_Window
            ) -> &'a $ty {
                &window.data::<Self>(tree).$name
            }

            $vis fn [< $name _mut >] <T>(
                tree: &mut $crate::tuifw_window_WindowTree,
                window: $crate::tuifw_window_Window,
                f: impl FnOnce(&mut $ty) -> T
            ) -> T {
                let value = &mut window.data_mut::<Self>(tree).$name;
                let res = f(value);
                window.invalidate_render(tree);
                $(Self::$on_changed(tree, window);)?
                res
            }

            $vis fn [< set_ $name >] <'a>(
                tree: &mut $crate::tuifw_window_WindowTree,
                window: $crate::tuifw_window_Window,
                value: impl Into<$crate::alloc_borrow_Cow<'a, <$ty as $crate::core_ops_Deref>::Target>>
            ) {
                Self:: [< $name _mut >] (
                    tree,
                    window,
                    |x| $crate::core_mem_replace(x, value.into().into_owned())
                );
            }
        }
    };
    (
        #[property(ref $(, changed=$on_changed:ident)?)]
        $vis:vis $name:ident : $ty:ty
    ) => {
        $crate::paste_paste! {
            $vis fn $name<'a>(
                tree: &'a $crate::tuifw_window_WindowTree,
                window: $crate::tuifw_window_Window
            ) -> &'a $ty {
                &window.data::<Self>(tree).$name
            }

            $vis fn [< $name _mut >] <T>(
                tree: &mut $crate::tuifw_window_WindowTree,
                window: $crate::tuifw_window_Window,
                f: impl FnOnce(&mut $ty) -> T
            ) -> T {
                let value = &mut window.data_mut::<Self>(tree).$name;
                let res = f(value);
                $(Self::$on_changed(tree, window);)?
                res
            }

            $vis fn [< set_ $name >] <'a>(
                tree: &mut $crate::tuifw_window_WindowTree,
                window: $crate::tuifw_window_Window,
                value: impl Into<$crate::alloc_borrow_Cow<'a, <$ty as $crate::core_ops_Deref>::Target>>
            ) {
                Self:: [< $name _mut >] (
                    tree,
                    window,
                    |x| $crate::core_mem_replace(x, value.into().into_owned())
                );
            }
        }
    };
    (
        #[property(obj, measure $(, changed=$on_changed:ident)?)]
        $vis:vis $name:ident : $ty:ty
    ) => {
        $crate::paste_paste! {
            $vis fn $name<'a>(
                tree: &'a $crate::tuifw_window_WindowTree,
                window: $crate::tuifw_window_Window
            ) -> &'a $ty {
                &window.data::<Self>(tree).$name
            }

            $vis fn [< $name _mut >] <T>(
                tree: &mut $crate::tuifw_window_WindowTree,
                window: $crate::tuifw_window_Window,
                f: impl FnOnce(&mut $ty) -> T
            ) -> T {
                let value = &mut window.data_mut::<Self>(tree).$name;
                let res = f(value);
                window.invalidate_measure(tree);
                $(Self::$on_changed(tree, window);)?
                res
            }

            $vis fn [< set_ $name >] (
                tree: &mut $crate::tuifw_window_WindowTree,
                window: $crate::tuifw_window_Window,
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
    (
        #[property(obj, render $(, changed=$on_changed:ident)?)]
        $vis:vis $name:ident : $ty:ty
    ) => {
        $crate::paste_paste! {
            $vis fn $name<'a>(
                tree: &'a $crate::tuifw_window_WindowTree,
                window: $crate::tuifw_window_Window
            ) -> &'a $ty {
                &window.data::<Self>(tree).$name
            }

            $vis fn [< $name _mut >] <T>(
                tree: &mut $crate::tuifw_window_WindowTree,
                window: $crate::tuifw_window_Window,
                f: impl FnOnce(&mut $ty) -> T
            ) -> T {
                let value = &mut window.data_mut::<Self>(tree).$name;
                let res = f(value);
                window.invalidate_render(tree);
                $(Self::$on_changed(tree, window);)?
                res
            }

            $vis fn [< set_ $name >] (
                tree: &mut $crate::tuifw_window_WindowTree,
                window: $crate::tuifw_window_Window,
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
    (
        #[property(obj $(, changed=$on_changed:ident)?)]
        $vis:vis $name:ident : $ty:ty
    ) => {
        $crate::paste_paste! {
            $vis fn $name<'a>(
                tree: &'a $crate::tuifw_window_WindowTree,
                window: $crate::tuifw_window_Window
            ) -> &'a $ty {
                &window.data::<Self>(tree).$name
            }

            $vis fn [< $name _mut >] <T>(
                tree: &mut $crate::tuifw_window_WindowTree,
                window: $crate::tuifw_window_Window,
                f: impl FnOnce(&mut $ty) -> T
            ) -> T {
                let value = &mut window.data_mut::<Self>(tree).$name;
                let res = f(value);
                $(Self::$on_changed(tree, window);)?
                res
            }

            $vis fn [< set_ $name >] (
                tree: &mut $crate::tuifw_window_WindowTree,
                window: $crate::tuifw_window_Window,
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
    (
        $(#[property($($attrs:tt)*)])?
        $vis:vis $field_name:ident : $field_ty:ty
    ) => {
        $crate::core_compile_error!($crate::core_concat!(
            "invalid widget property: ",
            $crate::core_stringify!(
                $(#[property($($attrs)*)])?
                $vis $field_name : $field_ty
            )
        ));
    };
}
