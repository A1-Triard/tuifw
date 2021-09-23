use components_arena::{Arena, Component, ComponentId, Id, NewtypeComponentId, RawId};
use debug_panic::debug_panic;
use dep_obj::{DepObjBaseBuilder, DepObjIdBase, DepType, dep_obj, dep_type_with_builder, DepEventArgs, Convenient, DepProp};
use dep_obj::binding::{Binding, Binding0, Binding1, Binding5, Bindings, b_immediate};
use downcast_rs::{Downcast, impl_downcast};
use dyn_clone::{DynClone, clone_trait_object};
use dyn_context::state::{SelfState, State, StateExt, StateRefMut, RequiresStateDrop, StateDrop};
use macro_attr_2018::macro_attr;
use std::any::Any;
use std::borrow::Cow;
use std::cmp::{max, min};
use std::fmt::Debug;
use std::mem::replace;
use std::num::NonZeroU16;
use tuifw_screen_base::{Attr, Color, Event, HAlign, Key, Point, Rect, Screen, Thickness, VAlign, Vector};
use tuifw_window::{RenderPort, Window, WindowTree};
use std::sync::{Arc, Mutex};

pub trait Layout: Downcast + DepType<Id=View> + Send + Sync {
    fn behavior(&self) -> &'static dyn LayoutBehavior;
}

impl_downcast!(Layout);

pub trait LayoutBehavior {
    fn init_bindings(&self, view: View, state: &mut dyn State) -> Box<dyn LayoutBindings>;

    fn drop_bindings(&self, view: View, state: &mut dyn State, bindings: Box<dyn LayoutBindings>);
}

pub trait LayoutBindings: Downcast + Debug + Send + Sync { }

impl_downcast!(LayoutBindings);

pub trait PanelBehavior {
    fn children_desired_size(
        &self,
        view: View,
        state: &mut dyn State,
        children_measure_size: (Option<i16>, Option<i16>)
    ) -> Vector;

    fn children_render_bounds(
        &self,
        view: View,
        state: &mut dyn State,
        children_arrange_bounds: Rect
    ) -> Rect;

    fn init_bindings(&self, view: View, state: &mut dyn State) -> Box<dyn PanelBindings>;

    fn drop_bindings(&self, view: View, state: &mut dyn State, bindings: Box<dyn PanelBindings>);
}

pub trait Panel: Downcast + DepType<Id=View> + Send + Sync {
    fn behavior(&self) -> &'static dyn PanelBehavior;
}

impl_downcast!(Panel);

pub trait PanelBindings: Downcast + Debug + Send + Sync { }

impl_downcast!(PanelBindings);

pub trait DecoratorBehavior {
    fn ty(&self) -> &'static str;

    fn children_measure_size(
        &self,
        view: View,
        state: &mut dyn State,
        measure_size: (Option<i16>, Option<i16>)
    ) -> (Option<i16>, Option<i16>);

    fn desired_size(&self, view: View, state: &mut dyn State, children_desired_size: Vector) -> Vector;

    fn children_arrange_bounds(&self, view: View, state: &mut dyn State, arrange_size: Vector) -> Rect;

    fn render_bounds(&self, view: View, state: &mut dyn State, children_render_bounds: Rect) -> Rect;

    fn render(&self, view: View, state: &dyn State, port: &mut RenderPort);

    fn init_bindings(&self, view: View, state: &mut dyn State) -> Box<dyn DecoratorBindings>;

    fn drop_bindings(&self, view: View, state: &mut dyn State, bindings: Box<dyn DecoratorBindings>);
}

pub trait DecoratorBindings: Downcast + Debug + Sync + Send { }

impl_downcast!(DecoratorBindings);

pub trait Decorator: Downcast + DepType<Id=View> + Sync + Send {
    fn behavior(&self) -> &'static dyn DecoratorBehavior;
}

impl_downcast!(Decorator);

#[derive(Debug, Clone, Eq, PartialEq)]
struct ViewSizeMinMax {
    min_size: Vector,
    max_w: Option<i16>,
    max_h: Option<i16>,
}

impl Default for ViewSizeMinMax {
    fn default() -> Self {
        ViewSizeMinMax { min_size: Vector::null(), max_w: None, max_h: None }
    }
}

#[derive(Debug, Clone)]
struct ViewRawAlignBindings {
    size_min_max: Binding<ViewSizeMinMax>,
    margin: Binding<Thickness>,
    h_align: Binding<HAlign>,
    v_align: Binding<VAlign>,
}

impl ViewRawAlignBindings {
    fn drop_bindings(self, state: &mut dyn State) {
        self.size_min_max.drop_binding(state);
        self.margin.drop_binding(state);
        self.h_align.drop_binding(state);
        self.v_align.drop_binding(state);
    }
}

macro_attr! {
    #[derive(Debug)]
    #[derive(Component!)]
    struct ViewNode {
        tag: Option<RawId>,
        decorator: Option<Box<dyn Decorator>>,
        decorator_bindings: Option<Box<dyn DecoratorBindings>>,
        window: Window,
        panel: Option<Box<dyn Panel>>,
        panel_bindings: Option<Box<dyn PanelBindings>>,
        layout: Option<Box<dyn Layout>>,
        layout_bindings: Option<Box<dyn LayoutBindings>>,
        base: ViewBase,
        align: Option<ViewAlign>,
        raw_align_bindings: Option<ViewRawAlignBindings>,
        measure_size: Option<(Option<i16>, Option<i16>)>,
        desired_size: Vector,
        arrange_bounds: Option<Rect>,
        render_bounds: Rect,
    }
}

#[derive(Debug)]
pub struct ViewTree(StateDrop<ViewTreeImpl>);

#[derive(Debug)]
struct ViewTreeImpl {
    arena: Arena<ViewNode>,
    window_tree: Option<WindowTree>,
    screen_size: Vector,
    root: View,
    focused: View,
    actual_focused: View,
    quit: bool,
    update_actual_focused_queue: usize,
    update_actual_focused_enqueue: bool,
}

impl SelfState for ViewTree { }

impl RequiresStateDrop for ViewTreeImpl {
    fn get(state: &dyn State) -> &StateDrop<Self> {
        let tree: &ViewTree = state.get();
        &tree.0
    }

    fn get_mut(state: &mut dyn State) -> &mut StateDrop<Self> {
        let tree: &mut ViewTree = state.get_mut();
        &mut tree.0
    }

    fn before_drop(state: &mut dyn State) {
        let tree: &ViewTree = state.get();
        let root = tree.0.get().root;
        root.drop_bindings_tree(state);
    }

    fn drop_incorrectly(self) {
        debug_panic!("ViewTree should be dropped with the drop_self method");
    }
}

impl ViewTree {
    pub fn new(
        screen: Box<dyn Screen>,
        bindings: &mut Bindings,
    ) -> ViewTree {
        let mut arena = Arena::new();
        let (window_tree, root, decorator_behavior) = arena.insert(|view| {
            let mut window_tree = WindowTree::new(screen, render_view);
            let window = Window::new(&mut window_tree, None, None, Rect { tl: Point { x: 0, y: 0 }, size: Vector::null() });
            window.set_tag(&mut window_tree, view);
            let screen_size = window_tree.screen_size();
            let decorator = RootDecorator::new_priv();
            let decorator_behavior = decorator.behavior();
            (ViewNode {
                tag: None,
                base: ViewBase::new_priv(),
                align: None,
                decorator: Some(Box::new(decorator)),
                decorator_bindings: None,
                window,
                layout: None,
                layout_bindings: None,
                panel: None,
                panel_bindings: None,
                raw_align_bindings: None,
                measure_size: Some((Some(screen_size.x), Some(screen_size.y))),
                desired_size: screen_size,
                arrange_bounds: Some(Rect { tl: Point { x: 0, y: 0 }, size: screen_size }),
                render_bounds: Rect { tl: Point { x: 0, y: 0 }, size: screen_size },
            }, (window_tree, View(view), decorator_behavior))
        });
        let screen_size = window_tree.screen_size();
        let mut tree = ViewTree(StateDrop::new(ViewTreeImpl {
            arena,
            window_tree: Some(window_tree),
            screen_size,
            root,
            focused: root,
            actual_focused: root,
            quit: false,
            update_actual_focused_queue: 0,
            update_actual_focused_enqueue: false,
        }));
        bindings.merge_mut_and_then(|state| {
            let size_min_max = Binding0::new(state, (), |()| Some(ViewSizeMinMax {
                min_size: Vector::null(),
                max_w: None,
                max_h: None,
            }));
            let margin = Binding0::new(state, (), |()| Some(Thickness::all(0)));
            let h_align = Binding0::new(state, (), |()| Some(HAlign::Left));
            let v_align = Binding0::new(state, (), |()| Some(VAlign::Top));
            {
                let tree: &mut ViewTree = state.get_mut();
                tree.0.get_mut().arena[root.0].raw_align_bindings = Some(ViewRawAlignBindings {
                    size_min_max: size_min_max.into(),
                    margin: margin.into(),
                    h_align: h_align.into(),
                    v_align: v_align.into(),
                });
            }
            let decorator_bindings = decorator_behavior.init_bindings(root, state);
            {
                let tree: &mut ViewTree = state.get_mut();
                let ok = tree.0.get_mut().arena[root.0].decorator_bindings.replace(decorator_bindings).is_none();
                debug_assert!(ok);
            }
        }, &mut tree);
        tree
    }

    pub fn drop_self(state: &mut dyn State) {
        <StateDrop<ViewTreeImpl>>::drop_self(state);
    }

    pub fn quit(state: &mut dyn State) {
        let tree: &mut ViewTree = state.get_mut();
        tree.0.get_mut().quit = true;
    }

    fn window_tree(&self) -> &WindowTree {
        self.0.get().window_tree.as_ref().expect("ViewTree is in invalid state")
    }

    fn window_tree_mut(&mut self) -> &mut WindowTree {
        self.0.get_mut().window_tree.as_mut().expect("ViewTree is in invalid state")
    }

    pub fn root(&self) -> View { self.0.get().root }

    pub fn update(state: &mut dyn State, wait: bool) -> Result<bool, Box<dyn Any>> {
        let tree: &ViewTree = state.get();
        if tree.0.get().quit { return Ok(false); }
        Self::update_actual_focused(state);
        let tree: &ViewTree = state.get();
        let screen_size = tree.0.get().screen_size;
        let root = tree.0.get().root;
        root.measure(state, (Some(screen_size.x), Some(screen_size.y)));
        root.arrange(state, Rect { tl: Point { x: 0, y: 0 }, size: screen_size });
        let mut window_tree = {
            let tree: &mut ViewTree = state.get_mut();
            tree.0.get_mut().window_tree.take().expect("ViewTree is in invalid state")
        };
        let event = window_tree.update(wait, state);
        if let Ok(event) = &event {
            if event == &Some(Event::Resize) {
                let tree: &mut ViewTree = state.get_mut();
                tree.0.get_mut().screen_size = window_tree.screen_size();
            }
        }
        {
            let tree: &mut ViewTree = state.get_mut();
            tree.0.get_mut().window_tree.replace(window_tree);
        }
        let event = event?;
        if let Some(Event::Key(n, key)) = event {
            let input = ViewInput(Arc::new(Mutex::new(ViewInputInstance { key: (n, key), handled: false })));
            let tree: &ViewTree = state.get();
            let view = tree.0.get().actual_focused;
            b_immediate(ViewBase::INPUT.raise(state, view.base(), input));
        }
        Ok(true)
    }

    pub fn focused(&self) -> View { self.0.get().focused }

    fn update_actual_focused(state: &mut dyn State) {
        {
            let tree: &mut ViewTree = state.get_mut();
            if replace(&mut tree.0.get_mut().update_actual_focused_enqueue, true) {
                let update_actual_focused_queue = &mut tree.0.get_mut().update_actual_focused_queue;
                *update_actual_focused_queue = update_actual_focused_queue.checked_add(1).unwrap();
                return;
            }
        }
        loop {
            let focused;
            let actual_focused;
            {
                let tree: &mut ViewTree = state.get_mut();
                focused = tree.0.get().focused;
                actual_focused = tree.0.get().actual_focused;
                if focused == actual_focused { break; }
                tree.0.get_mut().actual_focused = focused;
            }
            let mut view = actual_focused;
            loop {
                b_immediate(ViewBase::IS_FOCUSED.set(state, view.base(), false));
                let tree: &ViewTree = state.get();
                if let Some(parent) = view.parent(tree) {
                    view = parent;
                } else {
                    break;
                }
            }
            let mut view = focused;
            loop {
                b_immediate(ViewBase::IS_FOCUSED.set(state, view.base(), true));
                let tree: &ViewTree = state.get();
                if let Some(parent) = view.parent(tree) {
                    view = parent;
                } else {
                    break;
                }
            }
            let tree: &mut ViewTree = state.get_mut();
            let update_actual_focused_queue = &mut tree.0.get_mut().update_actual_focused_queue;
            if *update_actual_focused_queue == 0 { break; }
            *update_actual_focused_queue -= 1;
        }
        {
            let tree: &mut ViewTree = state.get_mut();
            tree.0.get_mut().update_actual_focused_enqueue = false;
        }
    }
}

fn render_view(
    tree: &WindowTree,
    window: Option<Window>,
    port: &mut RenderPort,
    state: &mut dyn State,
) {
    let view_tree: &ViewTree = state.get();
    let view: View = window
        .map(|window| window.tag(tree).expect("Window is not bound to a View"))
        .unwrap_or(view_tree.0.get().root);
    view_tree.0.get().arena[view.0].decorator.as_ref().map(|decorator|
        decorator.behavior().render(view, state, port)
    );
}

pub struct ViewBuilder<'a> {
    view: View,
    state: &'a mut dyn State,
}

impl<'a> DepObjBaseBuilder<View> for ViewBuilder<'a> {
    fn id(&self) -> View { self.view }
    fn state(&self) -> &dyn State { self.state }
    fn state_mut(&mut self) -> &mut dyn State { self.state }
}

macro_attr! {
    #[derive(NewtypeComponentId!)]
    #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
    pub struct View(Id<ViewNode>);
}

impl View {
    pub fn new(
        state: &mut dyn State,
        parent: View,
    ) -> View {
        let view = {
            let tree: &mut ViewTree = state.get_mut();
            let parent_window = tree.0.get().arena[parent.0].window;
            let window = Window::new(
                tree.window_tree_mut(),
                Some(parent_window),
                None,
                Rect { tl: Point { x: 0, y: 0 }, size: Vector::null() }
            );
            let view = tree.0.get_mut().arena.insert(|view| {
                (ViewNode {
                    tag: None,
                    base: ViewBase::new_priv(),
                    align: Some(ViewAlign::new_priv()),
                    decorator: None,
                    decorator_bindings: None,
                    window,
                    layout: None,
                    layout_bindings: None,
                    panel: None,
                    panel_bindings: None,
                    raw_align_bindings: None,
                    measure_size: Some((None, None)),
                    desired_size: Vector::null(),
                    arrange_bounds: Some(Rect { tl: Point { x: 0, y: 0 }, size: Vector::null() }),
                    render_bounds: Rect { tl: Point { x: 0, y: 0 }, size: Vector::null() },
                }, view)
            });
            let view = View(view);
            window.set_tag(tree.window_tree_mut(), view);
            view
        };
        let size_min_max = Binding5::new(state, (), |(),
            w: Option<i16>,
            h: Option<i16>,
            min_size: Vector,
            max_w,
            max_h
        | Some(ViewSizeMinMax {
            min_size: Vector { x: w.unwrap_or(min_size.x), y: h.unwrap_or(min_size.y) },
            max_w: w.or(max_w),
            max_h: h.or(max_h),
        }));
        size_min_max.set_target_fn(state, view, |state, view, _| view.invalidate_measure(state));
        size_min_max.set_source_1(state, &mut ViewAlign::W.value_source(view.align()));
        size_min_max.set_source_2(state, &mut ViewAlign::H.value_source(view.align()));
        size_min_max.set_source_3(state, &mut ViewAlign::MIN_SIZE.value_source(view.align()));
        size_min_max.set_source_4(state, &mut ViewAlign::MAX_W.value_source(view.align()));
        size_min_max.set_source_5(state, &mut ViewAlign::MAX_H.value_source(view.align()));
        let margin = Binding1::new(state, (), |(), margin| Some(margin));
        margin.set_target_fn(state, view, |state, view, _| view.invalidate_measure(state));
        margin.set_source_1(state, &mut ViewAlign::MARGIN.value_source(view.align()));
        let h_align = Binding1::new(state, (), |(), h_align| Some(h_align));
        h_align.set_target_fn(state, view, |state, view, _| view.invalidate_arrange(state));
        h_align.set_source_1(state, &mut ViewAlign::H_ALIGN.value_source(view.align()));
        let v_align = Binding1::new(state, (), |(), v_align| Some(v_align));
        v_align.set_target_fn(state, view, |state, view, _| view.invalidate_arrange(state));
        v_align.set_source_1(state, &mut ViewAlign::V_ALIGN.value_source(view.align()));
        {
            let tree: &mut ViewTree = state.get_mut();
            tree.0.get_mut().arena[view.0].raw_align_bindings = Some(ViewRawAlignBindings {
                size_min_max: size_min_max.into(),
                margin: margin.into(),
                h_align: h_align.into(),
                v_align: v_align.into(),
            });
        }
        view
    }

    fn drop_bindings_tree(self, state: &mut dyn State) {
        self.drop_bindings(state);
        let tree: &ViewTree = state.get();
        if let Some(last_child) = self.last_child(tree) {
            let mut child = last_child;
            loop {
                let tree: &ViewTree = state.get();
                child = child.next(tree);
                child.drop_bindings_tree(state);
                if child == last_child { break; }
            }
        }
    }

    fn drop_bindings(self, state: &mut dyn State) {
        self.drop_decorator_bindings(state);
        self.drop_layout_bindings(state);
        self.drop_panel_bindings(state);
        let raw_align_bindings = {
            let tree: &mut ViewTree = state.get_mut();
            tree.0.get_mut().arena[self.0].raw_align_bindings.take().unwrap()
        };
        raw_align_bindings.drop_bindings(state);
        self.drop_bindings_priv(state);
    }

    fn drop_children(window: Window, tree: &mut ViewTree) {
        if let Some(last_child) = window.last_child(tree.window_tree()) {
            let mut child = last_child;
            loop {
                let node = {
                    let view: View = child.tag(tree.window_tree()).unwrap();
                    tree.0.get_mut().arena.remove(view.0)
                };
                Self::drop_children(node.window, tree);
                child = node.window.next(tree.window_tree());
                if child == last_child { break; }
            }
        }
    }

    pub fn drop_view(self, state: &mut dyn State) {
        self.drop_bindings_tree(state);
        let tree: &mut ViewTree = state.get_mut();
        let node = tree.0.get_mut().arena.remove(self.0);
        Self::drop_children(node.window, tree);
        node.window.drop_window(tree.window_tree_mut());
    }

    pub fn set_tag<Tag: ComponentId>(self, state: &mut dyn State, tag: Tag) {
        let tree: &mut ViewTree = state.get_mut();
        tree.0.get_mut().arena[self.0].tag = Some(tag.into_raw());
    }

    pub fn reset_tag(self, state: &mut dyn State) {
        let tree: &mut ViewTree = state.get_mut();
        tree.0.get_mut().arena[self.0].tag = None;
    }

    pub fn tag<Tag: ComponentId>(self, tree: &ViewTree) -> Option<Tag> {
        tree.0.get().arena[self.0].tag.map(Tag::from_raw)
    }

    pub fn build<'a>(
        self,
        state: &'a mut dyn State,
        f: impl FnOnce(ViewBuilder<'a>) -> ViewBuilder<'a>
    ) {
        f(ViewBuilder { view: self, state });
    }

    pub fn focus(self, state: &mut dyn State) -> View {
        let tree: &mut ViewTree = state.get_mut();
        replace(&mut tree.0.get_mut().focused, self)
    }

    fn drop_layout_bindings(self, state: &mut dyn State) {
        let bindings;
        let layout;
        {
            let tree: &mut ViewTree = state.get_mut();
            let node = &mut tree.0.get_mut().arena[self.0];
            bindings = node.layout_bindings.take();
            layout = node.layout.as_ref().map(|x| x.behavior());
        }
        if let Some(layout) = layout {
            layout.drop_bindings(self, state, bindings.unwrap());
        } else {
            assert!(bindings.is_none());
        }
    }

    fn drop_panel_bindings(self, state: &mut dyn State) {
        let bindings;
        let panel;
        {
            let tree: &mut ViewTree = state.get_mut();
            let node = &mut tree.0.get_mut().arena[self.0];
            bindings = node.panel_bindings.take();
            panel = node.panel.as_ref().map(|x| x.behavior());
        }
        if let Some(panel) = panel {
            panel.drop_bindings(self, state, bindings.unwrap());
        } else {
            assert!(bindings.is_none());
        }
    }

    fn drop_decorator_bindings(self, state: &mut dyn State) {
        let bindings;
        let decorator;
        {
            let tree: &mut ViewTree = state.get_mut();
            let node = &mut tree.0.get_mut().arena[self.0];
            bindings = node.decorator_bindings.take();
            decorator = node.decorator.as_ref().map(|x| x.behavior());
        }
        if let Some(decorator) = decorator {
            decorator.drop_bindings(self, state, bindings.unwrap());
        } else {
            debug_assert!(bindings.is_none());
        }
    }

    pub fn set_decorator<D: Decorator>(self, state: &mut dyn State, decorator: D) {
        let behavior = decorator.behavior();
        {
            let tree: &mut ViewTree = state.get_mut();
            assert!(tree.0.get_mut().arena[self.0].decorator.replace(Box::new(decorator)).is_none(), "Decorator is already set and cannot be changed");
            assert!(self.last_child(tree).is_none(), "Decorator should be set before attaching children");
            let render_bounds = self.render_bounds(tree);
            let window = tree.0.get().arena[self.0].window;
            window.move_xy(tree.window_tree_mut(), render_bounds);
        }
        let bindings = behavior.init_bindings(self, state);
        {
            let tree: &mut ViewTree = state.get_mut();
            let ok = tree.0.get_mut().arena[self.0].decorator_bindings.replace(bindings).is_none();
            debug_assert!(ok);
        }
    }

    pub fn set_layout<L: Layout>(self, state: &mut dyn State, layout: L) {
        let behavior = layout.behavior();
        {
            let tree: &mut ViewTree = state.get_mut();
            assert!(tree.0.get_mut().arena[self.0].layout.replace(Box::new(layout)).is_none(), "Layout is already set and cannot be changed");
        }
        let bindings = behavior.init_bindings(self, state);
        {
            let tree: &mut ViewTree = state.get_mut();
            tree.0.get_mut().arena[self.0].layout_bindings = Some(bindings);
        }
    }

    pub fn set_panel<P: Panel>(self, state: &mut dyn State, panel: P) {
        let behavior = panel.behavior();
        {
            let tree: &mut ViewTree = state.get_mut();
            assert!(tree.0.get_mut().arena[self.0].panel.replace(Box::new(panel)).is_none(), "Panel is already set and cannot be changed");
        }
        let bindings = behavior.init_bindings(self, state);
        {
            let tree: &mut ViewTree = state.get_mut();
            tree.0.get_mut().arena[self.0].panel_bindings = Some(bindings);
        }
    }

    pub fn decorator_bindings(self, tree: &ViewTree) -> &dyn DecoratorBindings {
        tree.0.get().arena[self.0].decorator_bindings.as_ref().expect("Decorator Bindings missing").as_ref()
    }

    pub fn layout_bindings(self, tree: &ViewTree) -> &dyn LayoutBindings {
        tree.0.get().arena[self.0].layout_bindings.as_ref().expect("Layout Bindings missing").as_ref()
    }

    pub fn panel_bindings(self, tree: &ViewTree) -> &dyn PanelBindings {
        tree.0.get().arena[self.0].panel_bindings.as_ref().expect("Panel Bindings missing").as_ref()
    }

    pub fn last_child(self, tree: &ViewTree) -> Option<View> {
        let window_tree = tree.window_tree();
        tree.0.get().arena[self.0].window.last_child(window_tree).map(|x| x.tag(window_tree).unwrap())
    }

    pub fn prev(self, tree: &ViewTree) -> View {
        let window_tree = tree.window_tree();
        tree.0.get().arena[self.0].window.prev(window_tree).tag(window_tree).unwrap()
    }

    pub fn next(self, tree: &ViewTree) -> View {
        let window_tree = tree.window_tree();
        tree.0.get().arena[self.0].window.next(window_tree).tag(window_tree).unwrap()
    }

    pub fn parent(self, tree: &ViewTree) -> Option<View> {
        let window_tree = tree.window_tree();
        tree.0.get().arena[self.0].window.parent(window_tree).map(|x| x.tag(window_tree).unwrap())
    }

    pub fn desired_size(self, tree: &ViewTree) -> Vector { tree.0.get().arena[self.0].desired_size }

    pub fn render_bounds(self, tree: &ViewTree) -> Rect { tree.0.get().arena[self.0].render_bounds }

    fn bind_raw<P: Clone + 'static, T: Convenient, U: Convenient>(
        self,
        state: &mut dyn State,
        view_base_prop: DepProp<ViewBase, T>,
        map: fn(T) -> U,
        param: P,
        bind: fn(&mut dyn State, P, View, Binding<U>)
    ) {
        let binding = Binding1::new(state, map, |map, value: T| Some(map(value)));
        bind(state, param, self, binding.into());
        binding.set_source_1(state, &mut view_base_prop.value_source(self.base()));
    }

    pub fn bind_decorator_to_base<D: Decorator, T: Convenient, U: Convenient>(
        self,
        state: &mut dyn State,
        decorator_prop: DepProp<D, U>,
        view_base_prop: DepProp<ViewBase, T>,
        map: fn(T) -> U,
    ) {
        self.bind_raw(state, view_base_prop, map, decorator_prop, |state, decorator_prop, view, binding|
            decorator_prop.bind(state, view.decorator(), binding)
        );
    }

    dep_obj! {
        pub fn base(self as this, tree: ViewTree) -> (ViewBase) {
            if mut {
                &mut tree.0.get_mut().arena[this.0].base
            } else {
                &tree.0.get().arena[this.0].base
            }
        }

        pub fn align(self as this, tree: ViewTree) -> optional(ViewAlign) {
            if mut {
                tree.0.get_mut().arena[this.0].align.as_mut()
            } else {
                tree.0.get().arena[this.0].align.as_ref()
            }
        }

        pub fn decorator(self as this, tree: ViewTree) -> optional(trait Decorator) {
            if mut {
                tree.0.get_mut().arena[this.0].decorator.as_deref_mut()
            } else {
                tree.0.get().arena[this.0].decorator.as_deref()
            }
        }

        pub fn layout(self as this, tree: ViewTree) -> optional(trait Layout) {
            if mut {
                tree.0.get_mut().arena[this.0].layout.as_deref_mut()
            } else {
                tree.0.get().arena[this.0].layout.as_deref()
            }
        }

        pub fn panel(self as this, tree: ViewTree) -> optional(trait Panel) {
            if mut {
                tree.0.get_mut().arena[this.0].panel.as_deref_mut()
            } else {
                tree.0.get().arena[this.0].panel.as_deref()
            }
        }
    }

    pub fn invalidate_rect(self, state: &mut dyn State, rect: Rect) {
        let tree: &mut ViewTree = state.get_mut();
        if self == tree.0.get().root { return tree.window_tree_mut().invalidate_rect(rect); }
        let window = tree.0.get().arena[self.0].window;
        window.invalidate_rect(tree.window_tree_mut(), rect);
    }

    pub fn invalidate_render(self, state: &mut dyn State) {
        let tree: &mut ViewTree = state.get_mut();
        if self == tree.0.get().root { return tree.window_tree_mut().invalidate_screen(); }
        let window = tree.0.get().arena[self.0].window;
        window.invalidate(tree.window_tree_mut());
    }

    pub fn invalidate_parent_measure(self, state: &mut dyn State) {
        let tree: &ViewTree = state.get();
        self.parent(tree).map(|parent| parent.invalidate_measure(state));
    }

    pub fn invalidate_measure_and_render(self, state: &mut dyn State) {
        self.invalidate_measure(state);
        self.invalidate_render(state);
    }

    pub fn invalidate_measure(self, state: &mut dyn State) {
        let tree: &mut ViewTree = state.get_mut();
        let mut view = self;
        loop {
            if replace(&mut tree.0.get_mut().arena[view.0].measure_size, None).is_none() {
                debug_assert!(tree.0.get().arena[view.0].arrange_bounds.is_none());
                break;
            }
            tree.0.get_mut().arena[view.0].arrange_bounds = None;
            if let Some(parent) = view.parent(tree) {
                view = parent;
            } else {
                break;
            }
        }
    }

    pub fn invalidate_parent_arrange(self, state: &mut dyn State) {
        let tree: &ViewTree = state.get();
        self.parent(tree).map(|parent| parent.invalidate_arrange(state));
    }

    pub fn invalidate_arrange(self, state: &mut dyn State) {
        let tree: &mut ViewTree = state.get_mut();
        let mut view = self;
        loop {
            if replace(&mut tree.0.get_mut().arena[view.0].arrange_bounds, None).is_none() {
                break;
            }
            if let Some(parent) = view.parent(tree) {
                view = parent;
            } else {
                break;
            }
        }
    }

    pub fn measure(self, state: &mut dyn State, mut size: (Option<i16>, Option<i16>)) {
        let tree: &mut ViewTree = state.get_mut();
        let node = &mut tree.0.get_mut().arena[self.0];
        if node.measure_size == Some(size) { return; }
        node.measure_size = Some(size);
        let raw_align_bindings = node.raw_align_bindings.as_ref().unwrap();
        let margin = raw_align_bindings.margin;
        let size_min_max = raw_align_bindings.size_min_max;
        let margin = margin.get_value(state).unwrap_or_default();
        let size_min_max = size_min_max.get_value(state).unwrap_or_default();
        size.0.as_mut().map(|w| *w = margin.shrink_band_w(*w));
        size.1.as_mut().map(|h| *h = margin.shrink_band_h(*h));
        size.0 = size.0.map_or(size_min_max.max_w, |w| {
            let w = max(w as u16, size_min_max.min_size.x as u16);
            Some(size_min_max.max_w.map_or(w, |max_w| min(w, max_w as u16)) as i16)
        });
        size.1 = size.1.map_or(size_min_max.max_h, |h| {
            let h = max(h as u16, size_min_max.min_size.y as u16);
            Some(size_min_max.max_h.map_or(h, |max_h| min(h, max_h as u16)) as i16)
        });
        let tree: &ViewTree = state.get();
        let node = &tree.0.get().arena[self.0];
        let panel = node.panel.as_ref().map(|x| x.behavior());
        let decorator = node.decorator.as_ref().map(|x| x.behavior());
        let children_measure_size = decorator.as_ref().map_or(
            size,
            |d| d.children_measure_size(self, state, size)
        );
        let children_desired_size = if let Some(panel) = panel.as_ref() {
            panel.children_desired_size(self, state, children_measure_size)
        } else {
            let tree: &ViewTree = state.get();
            if let Some(last_child) = self.last_child(tree) {
                let mut children_desired_size = Vector::null();
                let mut child = last_child;
                loop {
                    let tree: &ViewTree = state.get();
                    child = child.next(tree);
                    child.measure(state, children_measure_size);
                    let tree: &ViewTree = state.get();
                    children_desired_size = children_desired_size.max(child.desired_size(tree));
                    if child == last_child { break children_desired_size; }
                }
            } else {
                Vector::null()
            }
        };
        let mut desired_size = decorator.as_ref().map_or(
            children_desired_size,
            |d| d.desired_size(self, state, children_desired_size)
        );
        desired_size = size_min_max.min_size.max(desired_size);
        if let Some(max_w) = size_min_max.max_w {
            desired_size.x = min(desired_size.x as u16, max_w as u16) as i16;
        }
        if let Some(max_h) = size_min_max.max_h {
            desired_size.y = min(desired_size.y as u16, max_h as u16) as i16;
        }
        {
            let tree: &mut ViewTree = state.get_mut();
            let node = &mut tree.0.get_mut().arena[self.0];
            node.desired_size = margin.expand_rect_size(desired_size);
        }
    }

    pub fn arrange(self, state: &mut dyn State, mut rect: Rect) {
        let tree: &mut ViewTree = state.get_mut();
        let node = &mut tree.0.get_mut().arena[self.0];
        if let Some(arrange_bounds) = node.arrange_bounds.as_mut() {
            if arrange_bounds.size == rect.size {
                if rect.tl != arrange_bounds.tl {
                    node.render_bounds.tl = node.render_bounds.tl.offset(
                        rect.tl.offset_from(arrange_bounds.tl)
                    );
                    arrange_bounds.tl = rect.tl;
                    let render_bounds = node.render_bounds;
                    node.window.move_xy(tree.window_tree_mut(), render_bounds);
                }
                return;
            }
        }
        node.arrange_bounds = Some(rect);
        let panel = node.panel.as_ref().map(|x| x.behavior());
        let decorator = node.decorator.as_ref().map(|x| x.behavior());
        let raw_align_bindings = node.raw_align_bindings.clone().unwrap();
        let margin = raw_align_bindings.margin.get_value(state).unwrap_or_default();
        let size_min_max = raw_align_bindings.size_min_max.get_value(state).unwrap_or_default();
        let h_align = raw_align_bindings.h_align.get_value(state).unwrap_or(HAlign::Center);
        let v_align = raw_align_bindings.v_align.get_value(state).unwrap_or(VAlign::Center);
        rect = margin.shrink_rect(rect);
        let mut size = size_min_max.min_size.max(rect.size);
        if let Some(max_w) = size_min_max.max_w {
            size.x = min(size.x as u16, max_w as u16) as i16;
        }
        if let Some(max_h) = size_min_max.max_h {
            size.y = min(size.y as u16, max_h as u16) as i16;
        }
        let padding = Thickness::align(size, rect.size, h_align, v_align);
        rect = padding.shrink_rect(rect);
        let children_arrange_bounds = decorator.as_ref().map_or_else(
            || Rect { tl: Point { x: 0, y: 0 }, size: rect.size },
            |d| d.children_arrange_bounds(self, state, rect.size)
        );
        let children_render_bounds = if let Some(panel) = panel.as_ref() {
            panel.children_render_bounds(self, state, children_arrange_bounds)
        } else {
            let tree: &ViewTree = state.get();
            if let Some(last_child) = self.last_child(tree) {
                let mut children_render_bounds = Rect { tl: Point { x: 0, y: 0 }, size: Vector::null() };
                let mut child = last_child;
                loop {
                    let tree: &ViewTree = state.get();
                    child = child.next(tree);
                    child.arrange(state, children_arrange_bounds);
                    let tree: &ViewTree = state.get();
                    children_render_bounds = children_render_bounds.union_intersect(
                        child.render_bounds(tree),
                        children_arrange_bounds
                    );
                    if child == last_child { break children_render_bounds; }
                }
            } else {
                children_arrange_bounds
            }
        };
        let mut render_bounds = decorator.as_ref().map_or(
            children_render_bounds,
            |d| d.render_bounds(self, state, children_render_bounds)
        );
        let padding = Thickness::align(render_bounds.size, rect.size, h_align, v_align);
        render_bounds.tl = rect.tl.offset(Point { x: 0, y: 0 }.offset_from(padding.expand_rect(render_bounds).tl));
        {
            let tree: &mut ViewTree = state.get_mut();
            let window = tree.0.get().arena[self.0].window;
            window.move_xy(tree.window_tree_mut(), render_bounds);
            tree.0.get_mut().arena[self.0].render_bounds = render_bounds;
        }
    }
}

impl DepObjIdBase for View {
    fn parent(self, state: &dyn State) -> Option<Self> {
        let tree: &ViewTree = state.get();
        self.parent(tree)
    }

    fn next(self, state: &dyn State) -> Self {
        let tree: &ViewTree = state.get();
        self.next(tree)
    }

    fn last_child(self, state: &dyn State) -> Option<Self> {
        let tree: &ViewTree = state.get();
        self.last_child(tree)
    }
}

pub trait ViewBuilderRootDecoratorExt {
    fn root_decorator(
        self,
        f: impl for<'a> FnOnce(RootDecoratorBuilder<'a>) -> RootDecoratorBuilder<'a>
    ) -> Self;
}

impl<'a> ViewBuilderRootDecoratorExt for ViewBuilder<'a> {
    fn root_decorator(
        self,
        f: impl for<'b> FnOnce(RootDecoratorBuilder<'b>) -> RootDecoratorBuilder<'b>
    ) -> Self {
        f(RootDecoratorBuilder::new_priv(self)).base_priv()
    }
}

#[derive(Debug)]
struct RootDecoratorBindings {
    fg: Binding<Color>,
    bg: Binding<Option<Color>>,
    attr: Binding<Attr>,
    fill: Binding<Cow<'static, str>>,
}

impl DecoratorBindings for RootDecoratorBindings { }

dep_type_with_builder! {
    #[derive(Debug)]
    pub struct RootDecorator become decorator in View {
        fill: Cow<'static, str> = Cow::Borrowed(" ")
    }

    type BaseBuilder<'a> = ViewBuilder<'a>;
}

impl RootDecorator {
    const BEHAVIOR: RootDecoratorBehavior = RootDecoratorBehavior;
}

impl Decorator for RootDecorator {
    fn behavior(&self) -> &'static dyn DecoratorBehavior { &Self::BEHAVIOR }
}

struct RootDecoratorBehavior;

impl DecoratorBehavior for RootDecoratorBehavior {
    fn ty(&self) -> &'static str { "Root" }

    fn children_measure_size(
        &self,
        _view: View,
        _state: &mut dyn State,
        measure_size: (Option<i16>, Option<i16>)
    ) -> (Option<i16>, Option<i16>) {
        measure_size
    }

    fn desired_size(&self, _view: View, _state: &mut dyn State, children_desired_size: Vector) -> Vector {
        children_desired_size
    }

    fn children_arrange_bounds(&self, _view: View, _state: &mut dyn State, arrange_size: Vector) -> Rect {
        Rect { tl: Point { x: 0, y: 0 }, size: arrange_size }
    }

    fn render_bounds(&self, _view: View, state: &mut dyn State, _children_render_bounds: Rect) -> Rect {
        let tree: &ViewTree = state.get();
        Rect { tl: Point { x: 0, y: 0 }, size: tree.0.get().screen_size }
    }

    fn init_bindings(&self, view: View, state: &mut dyn State) -> Box<dyn DecoratorBindings> {
        let bg = Binding1::new(state, (), |(), bg| Some(bg));
        let fg = Binding1::new(state, (), |(), fg| Some(fg));
        let attr = Binding1::new(state, (), |(), attr| Some(attr));
        let fill = Binding1::new(state, (), |(), fill| Some(fill));
        bg.set_target_fn(state, view, |state, view, _| view.invalidate_render(state));
        fg.set_target_fn(state, view, |state, view, _| view.invalidate_render(state));
        attr.set_target_fn(state, view, |state, view, _| view.invalidate_render(state));
        fill.set_target_fn(state, view, |state, view, _| view.invalidate_render(state));
        bg.set_source_1(state, &mut ViewBase::BG.value_source(view.base()));
        fg.set_source_1(state, &mut ViewBase::FG.value_source(view.base()));
        attr.set_source_1(state, &mut ViewBase::ATTR.value_source(view.base()));
        fill.set_source_1(state, &mut RootDecorator::FILL.value_source(view.decorator()));
        Box::new(RootDecoratorBindings {
            bg: bg.into(),
            fg: fg.into(),
            attr: attr.into(),
            fill: fill.into(),
        })
    }

    fn drop_bindings(&self, _view: View, state: &mut dyn State, bindings: Box<dyn DecoratorBindings>) {
        let bindings = bindings.downcast::<RootDecoratorBindings>().unwrap();
        bindings.fg.drop_binding(state);
        bindings.bg.drop_binding(state);
        bindings.attr.drop_binding(state);
        bindings.fill.drop_binding(state);
    }

    fn render(&self, view: View, state: &dyn State, port: &mut RenderPort) {
        let tree: &ViewTree = state.get();
        let bindings = view.decorator_bindings(tree).downcast_ref::<RootDecoratorBindings>().unwrap();
        let fg = bindings.fg.get_value(state).unwrap_or(Color::White);
        let bg = bindings.bg.get_value(state).unwrap_or(None);
        let attr = bindings.attr.get_value(state).unwrap_or_else(Attr::empty);
        let fill = bindings.fill.get_value(state).unwrap_or(Cow::Borrowed(" "));
        port.fill(|port, p| port.out(p, fg, bg, attr, &fill));
    }
}

pub trait PanelTemplate: Debug + DynClone + Send + Sync {
    fn apply_panel(&self, state: &mut dyn State, view: View);
    fn apply_layout(&self, state: &mut dyn State, view: View);
}

clone_trait_object!(PanelTemplate);

#[derive(Debug)]
struct ViewInputInstance {
    key: (NonZeroU16, Key),
    handled: bool,
}

#[derive(Debug, Clone)]
pub struct ViewInput(Arc<Mutex<ViewInputInstance>>);

impl PartialEq for ViewInput {
    fn eq(&self, _other: &Self) -> bool { false }
}

impl ViewInput {
    pub fn key(&self) -> (NonZeroU16, Key) { self.0.lock().unwrap().key }

    pub fn mark_as_handled(&self) {
        self.0.lock().unwrap().handled = true;
    }
}

impl DepEventArgs for ViewInput {
    fn handled(&self) -> bool { self.0.lock().unwrap().handled }
}

pub trait ViewBuilderViewBaseExt {
    fn base(
        self,
        f: impl for<'a> FnOnce(ViewBaseBuilder<'a>) -> ViewBaseBuilder<'a>
    ) -> Self;
}

impl<'a> ViewBuilderViewBaseExt for ViewBuilder<'a> {
    fn base(
        self,
        f: impl for<'b> FnOnce(ViewBaseBuilder<'b>) -> ViewBaseBuilder<'b>
    ) -> Self {
        f(ViewBaseBuilder::new_priv(self)).base_priv()
    }
}

dep_type_with_builder! {
    #[derive(Debug)]
    pub struct ViewBase become base in View {
        #[inherits]
        fg: Color = Color::White,
        #[inherits]
        bg: Option<Color> = None,
        #[inherits]
        attr: Attr = Attr::empty(),
        is_focused: bool = false,
        #[bubble]
        input yield ViewInput,
    }

    type BaseBuilder<'a> = ViewBuilder<'a>;
}

pub trait ViewBuilderViewAlignExt {
    fn align(
        self,
        f: impl for<'a> FnOnce(ViewAlignBuilder<'a>) -> ViewAlignBuilder<'a>
    ) -> Self;
}

impl<'a> ViewBuilderViewAlignExt for ViewBuilder<'a> {
    fn align(
        self,
        f: impl for<'b> FnOnce(ViewAlignBuilder<'b>) -> ViewAlignBuilder<'b>
    ) -> Self {
        f(ViewAlignBuilder::new_priv(self)).base_priv()
    }
}

dep_type_with_builder! {
    #[derive(Debug)]
    pub struct ViewAlign become align in View {
        h_align: HAlign = HAlign::Center,
        v_align: VAlign = VAlign::Center,
        min_size: Vector = Vector::null(),
        max_w: Option<i16> = None,
        max_h: Option<i16> = None,
        w: Option<i16> = None,
        h: Option<i16> = None,
        margin: Thickness = Thickness::all(0),
    }

    type BaseBuilder<'a> = ViewBuilder<'a>;
}
