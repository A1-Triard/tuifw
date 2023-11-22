#![feature(start)]

#![windows_subsystem = "windows"]

#![deny(warnings)]

#![no_std]

extern crate alloc;
extern crate rlibc;

mod no_std {
    use composable_allocators::{AsGlobal, System};

    #[global_allocator]
    static ALLOCATOR: AsGlobal<System> = AsGlobal(System);

    #[panic_handler]
    fn panic_handler(info: &core::panic::PanicInfo) -> ! { panic_no_std::panic(info, b'P') }

    #[no_mangle]
    extern fn rust_eh_personality() { }
}

#[cfg(any(target_os="dos", windows))]
extern {
    type PEB;
}

#[cfg(all(not(target_os="dos"), not(windows)))]
#[start]
fn main(_: isize, _: *const *const u8) -> isize {
    start_and_print_err() as _
}

#[cfg(any(target_os="dos", windows))]
#[allow(non_snake_case)]
#[no_mangle]
extern "stdcall" fn mainCRTStartup(_: *const PEB) -> u64 {
    #[cfg(target_os="dos")]
    CodePage::load_or_exit_with_msg(99);
    start_and_print_err()
}

fn start_and_print_err() -> u64 {
    if let Err(e) = start() {
        libc_print::libc_eprintln!("{e}");
        1
    } else {
        0
    }
}

mod ui {
    include!(concat!(env!("OUT_DIR"), "/ui.rs"));
}

use alloc::boxed::Box;
use alloc::string::{String, ToString};
use timer_no_std::MonoClock;
use tuifw_screen::{Error, Key};
use tuifw_window::{Data, Event, EventHandler, Window, WindowTree, App};
use tuifw::{CheckBox, VirtItemsPresenter, CMD_VIRT_ITEMS_PRESENTER_BIND};

#[derive(Clone)]
struct Item {
    label: String,
}

impl Data for Item { }

struct State;

impl App for State { }

#[derive(Clone)]
struct RootEventHandler;

impl EventHandler for RootEventHandler {
    fn invoke(
        &self,
        tree: &mut WindowTree,
        _window: Window,
        event: Event,
        _event_source: Window,
        _state: &mut dyn App,
    ) -> bool {
        if event == Event::Key(Key::Escape) {
            tree.quit();
            true
        } else {
            false
        }
    }
}

#[derive(Clone)]
struct ItemsEventHandler;

impl EventHandler for ItemsEventHandler {
    fn invoke(
        &self,
        tree: &mut WindowTree,
        window: Window,
        event: Event,
        event_source: Window,
        _state: &mut dyn App,
    ) -> bool {
        if event == Event::Cmd(CMD_VIRT_ITEMS_PRESENTER_BIND) {
            let item_index = event_source.source_index(tree).unwrap();
            let item = VirtItemsPresenter::items(tree, window)[item_index].downcast_ref::<Item>().unwrap();
            let label = item.label.clone();
            CheckBox::set_text(tree, event_source, label);
            true
        } else {
            false
        }
    }
}

fn start() -> Result<(), Error> {
    let clock = unsafe { MonoClock::new() };
    let screen = unsafe { tuifw_screen::init(None, None) }?;
    let tree = &mut WindowTree::new(screen, &clock)?;
    let names = ui::build(tree)?;
    names.root.set_event_handler(tree, Some(Box::new(RootEventHandler)));
    names.items.set_event_handler(tree, Some(Box::new(ItemsEventHandler)));
    VirtItemsPresenter::items_mut(tree, names.items, |items| {
        items.push(Box::new(Item { label: "Item ~1~".to_string() }));
        items.push(Box::new(Item { label: "Item ~2~".to_string() }));
        items.push(Box::new(Item { label: "Item ~3~".to_string() }));
        items.push(Box::new(Item { label: "Item ~4~".to_string() }));
        items.push(Box::new(Item { label: "Item ~5~".to_string() }));
        items.push(Box::new(Item { label: "Item ~6~".to_string() }));
        items.push(Box::new(Item { label: "Item ~7~".to_string() }));
        items.push(Box::new(Item { label: "Item ~8~".to_string() }));
        items.push(Box::new(Item { label: "Item ~9~".to_string() }));
        items.push(Box::new(Item { label: "Item 1~0~".to_string() }));
        items.push(Box::new(Item { label: "Item 11".to_string() }));
        items.push(Box::new(Item { label: "Item 12".to_string() }));
        items.push(Box::new(Item { label: "Item 13".to_string() }));
        items.push(Box::new(Item { label: "Item 14".to_string() }));
        items.push(Box::new(Item { label: "Item 15".to_string() }));
    });
    VirtItemsPresenter::set_focus_first_item_primary(tree, names.items, true);
    let state = &mut State;
    tree.run(state)
}
