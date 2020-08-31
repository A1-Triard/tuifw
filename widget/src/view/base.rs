use std::any::TypeId;
use std::fmt::Debug;
use std::iter::{self};
use std::mem::{replace};
use std::num::{NonZeroUsize};
use boow::Bow;
use components_arena::{Id, Arena, ComponentClassMutex};
use dep_obj::reactive::{Context, ContextExt, Reactive};
use dep_obj::dep::{DepProp, DepObj, DepTypeBuilder, DepObjProps, DepTypeToken};
use downcast::Any;
use once_cell::sync::{self};
use tuifw_screen_base::{Event, Screen, Vector, Point, Rect, Attr, Color};
use tuifw_window::{RenderPort, WindowTree, Window};

pub trait PanelBehavior {
    fn children_desired_size(
        &self,
        view: View,
        tree: &mut ViewTree,
        children_measure_size: (Option<i16>, Option<i16>)
    ) -> Vector;

    fn children_render_bounds(
        &self,
        view: View,
        tree: &mut ViewTree,
        children_arrange_bounds: Rect
    ) -> Rect;
}

pub trait Panel: Any + DepObj + Debug + Send + Sync {
    fn behavior(&self) -> &'static dyn PanelBehavior;
}

downcast!(dyn Panel);

pub trait DecoratorBehavior {
    fn children_measure_size(
        &self,
        view: View,
        tree: &mut ViewTree,
        measure_size: (Option<i16>, Option<i16>)
    ) -> (Option<i16>, Option<i16>);

    fn desired_size(&self, view: View, tree: &mut ViewTree, children_desired_size: Vector) -> Vector;

    fn children_arrange_bounds(&self, view: View, tree: &mut ViewTree, arrange_size: Vector) -> Rect;

    fn render_bounds(&self, view: View, tree: &mut ViewTree, children_render_bounds: Rect) -> Rect;

    fn render(&self, view: View, tree: &ViewTree, port: &mut RenderPort);
}

pub trait Decorator: Any + DepObj + Debug + Sync + Send {
    fn behavior(&self) -> &'static dyn DecoratorBehavior;
}

downcast!(dyn Decorator);

macro_attr! {
    #[derive(Debug)]
    #[derive(Component!)]
    struct ViewNode {
        decorator: Option<Box<dyn Decorator>>,
        window: Option<Window<View, ViewTree>>,
        panel: Option<Box<dyn Panel>>,
        parent: Option<View>,
        next: View,
        last_child: Option<View>,
        measure_size: Option<(Option<i16>, Option<i16>)>,
        desired_size: Vector,
        arrange_bounds: Option<Rect>,
        render_bounds: Rect,
    }
}

static VIEW_NODE: ComponentClassMutex<ViewNode> = ComponentClassMutex::new();

#[derive(Debug)]
pub struct ViewTree {
    arena: Arena<ViewNode>,
    window_tree: Option<WindowTree<View, ViewTree>>,
    screen_size: Vector,
    root: View,
}

impl Context for ViewTree {
    fn get_raw(&self, type_: TypeId) -> Option<&dyn std::any::Any> {
        if type_ == TypeId::of::<ViewTree>() {
            Some(self as _)
        } else {
            None
        }
    }

    fn get_mut_raw(&mut self, type_: TypeId) -> Option<&mut dyn std::any::Any> {
        if type_ == TypeId::of::<ViewTree>() {
            Some(self as _)
        } else {
            None
        }
    }
}

impl ViewTree {
    pub fn new(screen: Box<dyn Screen>) -> Self {
        let mut arena = Arena::new(&mut VIEW_NODE.lock().unwrap());
        let (window_tree, root) = arena.insert(|view| {
            let window_tree = WindowTree::new(screen, render_view, View(view));
            let screen_size = window_tree.screen_size();
            let decorator = RootDecorator { dep_props: DepObjProps::new(ROOT_DECORATOR_TYPE.token()) };
            (ViewNode {
                decorator: Some(Box::new(decorator) as _),
                window: None,
                panel: None,
                parent: None,
                next: View(view),
                last_child: None,
                measure_size: Some((Some(screen_size.x), Some(screen_size.y))),
                desired_size: screen_size,
                arrange_bounds: Some(Rect { tl: Point { x: 0, y: 0 }, size: screen_size }),
                render_bounds: Rect { tl: Point { x: 0, y: 0 }, size: screen_size },
            }, (window_tree, View(view)))
        });
        let screen_size = window_tree.screen_size();
        let mut tree = ViewTree {
            arena,
            window_tree: Some(window_tree),
            screen_size,
            root,
        };
        root.decorator_on_changed(&mut tree, ROOT_DECORATOR_TYPE.bg(), RootDecorator::invalidate_bg);
        tree
    }

    fn window_tree(&mut self) -> &mut WindowTree<View, ViewTree> {
        self.window_tree.as_mut().expect("ViewTree is in invalid state")
    }

    pub fn root(&self) -> View { self.root }

    pub fn update(&mut self, wait: bool) -> Result<Option<Event>, Box<dyn std::any::Any>> {
        self.root.measure(self, (Some(self.screen_size.x), Some(self.screen_size.y)));
        let mut window_tree = self.window_tree.take().expect("ViewTree is in invalid state");
        let result = window_tree.update(wait, self);
        if let Ok(result) = &result {
            if result == &Some(Event::Resize) {
                self.screen_size = window_tree.screen_size();
            }
        }
        self.window_tree.replace(window_tree);
        result
    }
}

fn render_view(
    _tree: &WindowTree<View, ViewTree>,
    _window: Option<Window<View, ViewTree>>,
    port: &mut RenderPort,
    tag: &View,
    context: &mut ViewTree
) {
    context.arena[tag.0].decorator.as_ref().unwrap().behavior().render(*tag, context, port);
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct View(Id<ViewNode>);

impl View {
    pub fn new<T>(
        tree: &mut ViewTree,
        parent: View,
        decorator_and_panel: impl FnOnce(View) -> (
            Option<Box<dyn Decorator>>,
            Option<Box<dyn Panel>>,
            T
        )
    ) -> T {
        let parent_window = parent
            .self_and_parents(tree)
            .find_map(|view| tree.arena[view.0].window.as_ref().map(|x| *x))
        ;
        let arena = &mut tree.arena;
        let window_tree = tree.window_tree.as_mut().expect("ViewTree is in invalid state");
        let (view, result) = arena.insert(|view| {
            let (decorator, panel, result) = decorator_and_panel(View(view));
            let window = if decorator.is_none() {
                None
            } else {
                Some(Window::new(
                    window_tree,
                    parent_window,
                    Rect { tl: Point { x: 0, y: 0 }, size: Vector::null() },
                    |window| (View(view), window)
                ))
            };
            (ViewNode {
                decorator,
                window,
                panel,
                parent: Some(parent),
                next: View(view),
                last_child: None,
                measure_size: Some((None, None)),
                desired_size: Vector::null(),
                arrange_bounds: Some(Rect { tl: Point { x: 0, y: 0 }, size: Vector::null() }),
                render_bounds: Rect { tl: Point { x: 0, y: 0 }, size: Vector::null() },
            }, (view, result))
        });
        View(view).invalidate_measure(tree);
        result
    }

    pub unsafe fn from_raw_parts(raw_parts: (usize, NonZeroUsize)) -> Self {
        View(Id::from_raw_parts(raw_parts))
    }

    pub fn into_raw_parts(self) -> (usize, NonZeroUsize) {
        self.0.into_raw_parts()
    }

    pub fn parent(self, tree: &ViewTree) -> Option<View> { tree.arena[self.0].parent }

    pub fn self_and_parents<'a>(self, tree: &'a ViewTree) -> impl Iterator<Item=View> + 'a {
        let mut view = Some(self);
        iter::from_fn(move || {
            let parent = view.and_then(|view| view.parent(tree));
            replace(&mut view, parent)
        })
    }

    pub fn last_child(self, tree: &ViewTree) -> Option<View> { tree.arena[self.0].last_child }

    pub fn next(self, tree: &ViewTree) -> View { tree.arena[self.0].next }

    pub fn children<'a>(self, tree: &'a ViewTree) -> impl Iterator<Item=View> + 'a {
        let last_child = self.last_child(tree);
        let mut view = last_child;
        iter::from_fn(move || {
            view = view.map(|view| view.next(tree));
            if view == last_child { view = None; }
            view
        })
    }

    pub fn desired_size(self, tree: &ViewTree) -> Vector { tree.arena[self.0].desired_size }

    pub fn render_bounds(self, tree: &ViewTree) -> Rect { tree.arena[self.0].render_bounds }

    pub fn decorator_get<D: Decorator, T>(
        self,
        tree: &ViewTree,
        prop: DepProp<D, Reactive<View, T>>,
    ) -> &T {
        let decorator = tree.arena[self.0]
            .decorator
            .as_ref()
            .expect("Decorator missed")
            .downcast_ref::<D>()
            .expect("invalid cast")
        ;
        prop.get(decorator.dep_props()).get()
    }

    pub fn decorator_set_uncond<D: Decorator, T>(
        self,
        context: &mut dyn Context,
        prop: DepProp<D, Reactive<View, T>>,
        value: T,
    ) -> T {
        let tree = context.get_mut::<ViewTree>().expect("ViewTree required");
        let decorator = tree.arena[self.0]
            .decorator
            .as_mut()
            .expect("Decorator missed")
            .downcast_mut::<D>()
            .expect("invalid cast")
        ;
        let (old, on_changed) = prop.get_mut(decorator.dep_props_mut()).set_uncond(value);
        on_changed.raise(self, context, &old);
        old
    }

    pub fn decorator_set_distinct<D: Decorator, T: Eq>(
        self,
        context: &mut dyn Context,
        prop: DepProp<D, Reactive<View, T>>,
        value: T,
    ) -> T {
        let tree = context.get_mut::<ViewTree>().expect("ViewTree required");
        let decorator = tree.arena[self.0]
            .decorator
            .as_mut()
            .expect("Decorator missed")
            .downcast_mut::<D>()
            .expect("invalid cast")
        ;
        let (old, on_changed) = prop.get_mut(decorator.dep_props_mut()).set_distinct(value);
        on_changed.raise(self, context, &old);
        old
    }

    pub fn decorator_on_changed<D: Decorator, T>(
        self,
        tree: &mut ViewTree,
        prop: DepProp<D, Reactive<View, T>>,
        on_changed: fn(owner: View, context: &mut dyn Context, old: &T),
    ) {
        let decorator = tree.arena[self.0]
            .decorator
            .as_mut()
            .expect("Decorator missed")
            .downcast_mut::<D>()
            .expect("invalid cast")
        ;
        prop.get_mut(decorator.dep_props_mut()).on_changed(on_changed);
    }

    pub fn panel_get<P: Panel, T>(
        self,
        tree: &ViewTree,
        prop: DepProp<P, Reactive<View, T>>,
    ) -> &T {
        let panel = tree.arena[self.0]
            .panel
            .as_ref()
            .expect("Panel missed")
            .downcast_ref::<P>()
            .expect("invalid cast")
        ;
        prop.get(panel.dep_props()).get()
    }

    pub fn panel_set_uncond<P: Panel, T>(
        self,
        context: &mut dyn Context,
        prop: DepProp<P, Reactive<View, T>>,
        value: T,
    ) -> T {
        let tree = context.get_mut::<ViewTree>().expect("ViewTree required");
        let panel = tree.arena[self.0]
            .panel
            .as_mut()
            .expect("Panel missed")
            .downcast_mut::<P>()
            .expect("invalid cast")
        ;
        let (old, on_changed) = prop.get_mut(panel.dep_props_mut()).set_uncond(value);
        on_changed.raise(self, context, &old);
        old
    }

    pub fn panel_set_distinct<P: Panel, T: Eq>(
        self,
        context: &mut dyn Context,
        prop: DepProp<P, Reactive<View, T>>,
        value: T,
    ) -> T {
        let tree = context.get_mut::<ViewTree>().expect("ViewTree required");
        let panel = tree.arena[self.0]
            .panel
            .as_mut()
            .expect("Panel missed")
            .downcast_mut::<P>()
            .expect("invalid cast")
        ;
        let (old, on_changed) = prop.get_mut(panel.dep_props_mut()).set_distinct(value);
        on_changed.raise(self, context, &old);
        old
    }

    pub fn panel_on_changed<P: Panel, T>(
        self,
        tree: &mut ViewTree,
        prop: DepProp<P, Reactive<View, T>>,
        on_changed: fn(owner: View, context: &mut dyn Context, old: &T),
    ) {
        let panel = tree.arena[self.0]
            .panel
            .as_mut()
            .expect("Panel missed")
            .downcast_mut::<P>()
            .expect("invalid cast")
        ;
        prop.get_mut(panel.dep_props_mut()).on_changed(on_changed);
    }

    #[must_use]
    pub fn invalidate_rect(self, tree: &mut ViewTree, rect: Rect) -> Option<()> {
        if self == tree.root { return Some(tree.window_tree().invalidate_rect(rect)); }
        let window = tree.arena[self.0].window;
        window.map(|window| window.invalidate_rect(&mut tree.window_tree(), rect))
    }

    #[must_use]
    pub fn invalidate_render(self, tree: &mut ViewTree) -> Option<()> {
        if self == tree.root { return Some(tree.window_tree().invalidate_screen()); }
        let window = tree.arena[self.0].window;
        window.map(|window| window.invalidate(&mut tree.window_tree()))
    }
    
    pub fn invalidate_measure(self, tree: &mut ViewTree) {
        let mut view = self;
        loop {
            if replace(&mut tree.arena[view.0].measure_size, None).is_none() {
                debug_assert!(tree.arena[view.0].arrange_bounds.is_none());
                break;
            }
            tree.arena[view.0].arrange_bounds = None;
            if let Some(parent) = view.parent(tree) {
                view = parent;
            } else {
                break;
            }
        }
    }

    pub fn invalidate_arrange(self, tree: &mut ViewTree) {
        let mut view = self;
        loop {
            if replace(&mut tree.arena[view.0].arrange_bounds, None).is_none() {
                break;
            }
            if let Some(parent) = view.parent(tree) {
                view = parent;
            } else {
                break;
            }
        }
    }

    pub fn measure(self, tree: &mut ViewTree, size: (Option<i16>, Option<i16>)) {
        let node = &mut tree.arena[self.0];
        if node.measure_size == Some(size) { return; }
        node.measure_size = Some(size);
        let panel = node.panel.as_ref().map(|x| x.behavior());
        let decorator = node.decorator.as_ref().map(|x| x.behavior());
        let children_measure_size = decorator.as_ref().map_or(
            size,
            |d| d.children_measure_size(self, tree, size)
        );
        let children_desired_size = if let Some(panel) = panel.as_ref() {
            panel.children_desired_size(self, tree, children_measure_size)
        } else {
            if let Some(last_child) = self.last_child(tree) {
                let mut children_desired_size = Vector::null();
                let mut child = last_child;
                loop {
                    child = child.next(tree);
                    if child == last_child { break children_desired_size; }
                    child.measure(tree, children_measure_size);
                    children_desired_size = children_desired_size.max(child.desired_size(tree));
                }
            } else {
                Vector::null()
            }
        };
        let desired_size = decorator.as_ref().map_or(
            children_desired_size,
            |d| d.desired_size(self, tree, children_desired_size)
        );
        let node = &mut tree.arena[self.0];
        node.desired_size = desired_size;
    }

    pub fn arrange(self, tree: &mut ViewTree, rect: Rect) {
        let node = &mut tree.arena[self.0];
        if let Some(arrange_bounds) = node.arrange_bounds.as_mut() {
            if arrange_bounds.size == rect.size {
                if rect.tl != arrange_bounds.tl {
                    node.render_bounds.tl = node.render_bounds.tl.offset(
                        rect.tl.offset_from(arrange_bounds.tl)
                    );
                    arrange_bounds.tl = rect.tl;
                    let render_bounds = node.render_bounds;
                    node.window.map(|w| w.move_(tree.window_tree(), render_bounds));
                }
                return;
            }
        }
        node.arrange_bounds = Some(rect);
        let panel = node.panel.as_ref().map(|x| x.behavior());
        let decorator = node.decorator.as_ref().map(|x| x.behavior());
        let children_arrange_bounds = decorator.as_ref().map_or_else(
            || Rect { tl: Point { x: 0, y: 0 }, size: rect.size },
            |d| d.children_arrange_bounds(self, tree, rect.size)
        );
        let children_render_bounds = if let Some(panel) = panel.as_ref() {
            panel.children_render_bounds(self, tree, children_arrange_bounds)
        } else {
            if let Some(last_child) = self.last_child(tree) {
                let mut children_render_bounds = Rect { tl: Point { x: 0, y: 0 }, size: Vector::null() };
                let mut child = last_child;
                loop {
                    child = child.next(tree);
                    if child == last_child { break children_render_bounds; }
                    child.arrange(tree, children_arrange_bounds);
                    children_render_bounds = children_render_bounds.union_intersect(
                        child.render_bounds(tree),
                        children_arrange_bounds
                    );
                }
            } else {
                children_arrange_bounds
            }
        };
        let render_bounds = decorator.as_ref().map_or(
            children_render_bounds,
            |d| d.render_bounds(self, tree, children_render_bounds)
        );
        let node = &mut tree.arena[self.0];
        node.render_bounds = Rect {
            tl: rect.tl.offset(render_bounds.tl.offset_from(Point { x: 0, y: 0 })),
            size: render_bounds.size
        }.intersect(rect);
    }
}

pub struct RootDecoratorType {
    token: DepTypeToken<RootDecorator>,
    bg: DepProp<RootDecorator, Reactive<View, Text>>,
}

impl RootDecoratorType {
    pub fn token(&self) -> &DepTypeToken<RootDecorator> { &self.token }
    pub fn bg(&self) -> DepProp<RootDecorator, Reactive<View, Text>> { self.bg }
}

pub static ROOT_DECORATOR_TYPE: sync::Lazy<RootDecoratorType> = sync::Lazy::new(|| {
    let mut builder = DepTypeBuilder::new().expect("RootDecoratorType builder locked");
    let bg = builder.prop::<Reactive<View, Text>>(|| Reactive::new(Text::SPACE.clone()));
    let token = builder.build();
    RootDecoratorType { token, bg }
});

macro_attr! {
    #[derive(DepObjRaw!)]
    #[derive(Debug)]
    pub struct RootDecorator {
        dep_props: DepObjProps<Self>,
    }
}

impl RootDecorator {
    fn invalidate_bg(_view: View, context: &mut dyn Context, _old: &Text) {
        let tree = context.get_mut::<ViewTree>().expect("ViewTree required");
        tree.window_tree().invalidate_screen();
    }
}

impl DepObj for RootDecorator {
    fn dep_props(&self) -> &DepObjProps<Self> { &self.dep_props }
    fn dep_props_mut(&mut self) -> &mut DepObjProps<Self> { &mut self.dep_props }
}

impl Decorator for RootDecorator {
    fn behavior(&self) -> &'static dyn DecoratorBehavior {
        static BEHAVIOR: RootDecoratorBehavior = RootDecoratorBehavior;
        &BEHAVIOR
    }
}

struct RootDecoratorBehavior;

impl DecoratorBehavior for RootDecoratorBehavior {
    fn children_measure_size(
        &self,
        _view: View,
        _tree: &mut ViewTree,
        measure_size: (Option<i16>, Option<i16>)
    ) -> (Option<i16>, Option<i16>) {
        measure_size
    }

    fn desired_size(&self, _view: View, _tree: &mut ViewTree, children_desired_size: Vector) -> Vector {
        children_desired_size
    }

    fn children_arrange_bounds(&self, _view: View, _tree: &mut ViewTree, arrange_size: Vector) -> Rect {
        Rect { tl: Point { x: 0, y: 0 }, size: arrange_size }
    }

    fn render_bounds(&self, _view: View, tree: &mut ViewTree, _children_render_bounds: Rect) -> Rect {
        Rect { tl: Point { x: 0, y: 0 }, size: tree.screen_size }
    }

    fn render(&self, view: View, tree: &ViewTree, port: &mut RenderPort) {
        let bg = view.decorator_get(tree, ROOT_DECORATOR_TYPE.bg());
        port.fill(|port, p| port.out(p, bg.fg, bg.bg, bg.attr, &bg.value));
    }
}

#[derive(Debug, Clone)]
pub struct Text {
    pub fg: Color,
    pub bg: Option<Color>,
    pub attr: Attr,
    pub value: Bow<'static, &'static str>,
}

impl Text {
    pub const SPACE: Text = Text {
        fg: Color::Black,
        bg: None,
        attr: Attr::empty(),
        value: Bow::Borrowed(&" ")
    };
}
