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

mod canvas;
pub use canvas::*;

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

mod content_presenter;
pub use content_presenter::*;

mod items_presenter;
pub use items_presenter::*;

mod virt_items_presenter;
pub use virt_items_presenter::*;

mod scroll_viewer;
pub use scroll_viewer::*;

pub mod virt_scroll_viewer;

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
            #[allow(clippy::new_ret_no_self)]
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
                $(Self::$init(tree, w)?;)?
                Ok(w)
            }

            $vis fn new_template(
                tree: &mut $crate::tuifw_window_WindowTree,
            ) -> Result<$crate::tuifw_window_Window, $crate::tuifw_screen_base_Error> {
                let w = $crate::tuifw_window_Window::new_template(
                    tree,
                    $crate::alloc_boxed_Box::new($Widget),
                )?;
                $(Self::$init(tree, w)?;)?
                Ok(w)
            }

            $($($crate::widget_impl! {
                @property
                $(#[property($($($attrs)*)?)])?
                $vis $field_name : $field_ty
            })+)?

            fn clone(
                #[allow(unused_variables)]
                tree: &mut $crate::tuifw_window_WindowTree,
                #[allow(unused_variables)]
                source: $crate::tuifw_window_Window,
                #[allow(unused_variables)]
                dest: $crate::tuifw_window_Window,
                #[allow(unused_variables)]
                clone_window: Box<dyn Fn(&WindowTree, Window) -> Window>,
            ) {
                $($($crate::widget_impl! {
                    @clone
                    $(#[property($($($attrs)*)?)])?
                    $field_name tree source dest clone_window : $field_ty
                })+)?
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! widget_impl {
    (
        @clone
        $name:ident $tree:ident $source:ident $dest:ident $clone_window:ident : $ty:ty
    ) => {
    };
    (
        @clone
        #[property(window $($x:tt)*)]
        $name:ident $tree:ident $source:ident $dest:ident $clone_window:ident : $ty:ty
    ) => {
        $crate::paste_paste! {
            Self:: [< set_ $name >] (
                $tree,
                $dest,
                Self::$name($tree, $source).map(|x| $clone_window($tree, x))
            );
        }
    };
    (
        @clone
        #[property(copy $($x:tt)*)]
        $name:ident $tree:ident $source:ident $dest:ident $clone_window:ident : $ty:ty
    ) => {
        $crate::paste_paste! {
            Self:: [< set_ $name >] ($tree, $dest, Self::$name($tree, $source));
        }
    };
    (
        @clone
        #[property($($x:tt)*)]
        $name:ident $tree:ident $source:ident $dest:ident $clone_window:ident : $ty:ty
    ) => {
        $crate::paste_paste! {
            Self:: [< set_ $name >] ($tree, $dest, <$ty as Clone>::clone(Self::$name($tree, $source)));
        }
    };
    (
        @property
        $vis:vis $field_name:ident : $field_ty:ty
    ) => {
    };
    (
        @property
        #[property(window, measure $(, on_changed=$on_changed:ident)?)]
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
        @property
        #[property(window, arrange $(, on_changed=$on_changed:ident)?)]
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
                window.invalidate_arrange(tree);
                $(Self::$on_changed(tree, window);)?
            }
        }
    };
    (
        @property
        #[property(window, render $(, on_changed=$on_changed:ident)?)]
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
        @property
        #[property(window $(, on_changed=$on_changed:ident)?)]
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
        @property
        #[property(copy, measure $(, on_changed=$on_changed:ident)?)]
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
        @property
        #[property(copy, arrange $(, on_changed=$on_changed:ident)?)]
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
                window.invalidate_arrange(tree);
                $(Self::$on_changed(tree, window);)?
            }
        }
    };
    (
        @property
        #[property(copy, render $(, on_changed=$on_changed:ident)?)]
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
        @property
        #[property(copy $(, on_changed=$on_changed:ident)?)]
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
        @property
        #[property(str, measure $(, on_changed=$on_changed:ident)?)]
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
        @property
        #[property(str, arrange $(, on_changed=$on_changed:ident)?)]
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
                window.invalidate_arrange(tree);
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
        @property
        #[property(str, render $(, on_changed=$on_changed:ident)?)]
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
        @property
        #[property(str $(, on_changed=$on_changed:ident)?)]
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
        @property
        #[property(ref, measure $(, on_changed=$on_changed:ident)?)]
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
        @property
        #[property(ref, arrange $(, on_changed=$on_changed:ident)?)]
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
                window.invalidate_arrange(tree);
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
        @property
        #[property(ref, render $(, on_changed=$on_changed:ident)?)]
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
        @property
        #[property(ref $(, on_changed=$on_changed:ident)?)]
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
        @property
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
