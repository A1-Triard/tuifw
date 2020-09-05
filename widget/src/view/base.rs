use std::any::TypeId;
use std::fmt::Debug;
use std::iter::{self};
use std::mem::{replace};
use std::num::{NonZeroUsize};
use boow::Bow;
use components_arena::{Id, Arena, ComponentClassMutex, ComponentId};
use dep_obj::{Context, ContextExt};
use dep_obj::{DepProp, DepObj, DepTypeToken};
use downcast::Any;
use once_cell::sync::{self};
use tuifw_screen_base::{Event, Screen, Vector, Point, Rect, Attr, Color};
use tuifw_window::{RenderPort, WindowTree, Window};

macro_attr! {
    #[derive(Eq, PartialEq, Debug, Hash, Clone, Copy, Ord, PartialOrd)]
    #[derive(EnumDisplay!, EnumFromStr!)]
    pub enum Orient {
        Hor,
        Vert
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

impl Default for Text {
    fn default() -> Text { Text::SPACE.clone() }
}

pub trait Layout: Any + Debug + Send + Sync { }

downcast!(dyn Layout);

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

pub trait Panel: Any + Debug + Send + Sync {
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

pub trait Decorator: Any + Debug + Sync + Send {
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
        layout: Option<Box<dyn Layout>>,
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
            let decorator = RootDecorator::new_raw(&ROOT_DECORATOR_TOKEN);
            (ViewNode {
                decorator: Some(Box::new(decorator) as _),
                window: None,
                layout: None,
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
        root.decorator_on_changed(&mut tree, root_decorator_type().bg(), RootDecorator::invalidate_bg);
        tree
    }

    fn window_tree(&mut self) -> &mut WindowTree<View, ViewTree> {
        self.window_tree.as_mut().expect("ViewTree is in invalid state")
    }

    pub fn root(&self) -> View { self.root }

    pub fn update(&mut self, wait: bool) -> Result<Option<Event>, Box<dyn std::any::Any>> {
        self.root.measure(self, (Some(self.screen_size.x), Some(self.screen_size.y)));
        self.root.arrange(self, Rect { tl: Point { x: 0, y: 0 }, size: self.screen_size });
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

macro_attr! {
    #[derive(ComponentId!)
    #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
    pub struct View(Id<ViewNode>);
}

impl View {
    pub fn new(
        tree: &mut ViewTree,
        parent: View,
    ) -> View {
        let arena = &mut tree.arena;
        let view = arena.insert(|view| {
            (ViewNode {
                decorator: None,
                window: None,
                layout: None,
                panel: None,
                parent: Some(parent),
                next: View(view),
                last_child: None,
                measure_size: Some((None, None)),
                desired_size: Vector::null(),
                arrange_bounds: Some(Rect { tl: Point { x: 0, y: 0 }, size: Vector::null() }),
                render_bounds: Rect { tl: Point { x: 0, y: 0 }, size: Vector::null() },
            }, view)
        });
        View(view).invalidate_measure(tree);
        View(view)
    }

    fn renew_window(self, tree: &mut ViewTree, parent_window: Option<Window<View, ViewTree>>) {
        let children_parent_window = if let Some(window) = tree.arena[self.0].window {
            window.drop(tree.window_tree());
            let render_bounds = self.render_bounds(tree);
            let window = Window::new(
                tree.window_tree(),
                parent_window,
                render_bounds,
                |window| (self, window)
            );
            tree.arena[self.0].window = Some(window);
            Some(window)
        } else {
            parent_window
        };
        if let Some(last_child) = self.last_child(tree) {
            let mut child = last_child;
            loop {
                child = child.next(tree);
                child.renew_window(tree, children_parent_window);
                if child == last_child { break; }
            }
        }
    }

    pub fn unset_decorator(self, tree: &mut ViewTree) {
        if self == tree.root { panic!("root view decorator can not be changed"); }
        if tree.arena[self.0].decorator.take().is_some() {
            let window = tree.arena[self.0].window.unwrap();
            window.drop(tree.window_tree());
            tree.arena[self.0].window = None;
            if let Some(last_child) = self.last_child(tree) {
                let parent_window = self
                    .self_and_parents(tree)
                    .find_map(|view| tree.arena[view.0].window)
                ;
                let mut child = last_child;
                loop {
                    child = child.next(tree);
                    child.renew_window(tree, parent_window);
                    if child == last_child { break; }
                }
            }
            self.invalidate_measure(tree);
        }
    }

    pub fn set_decorator<D: Decorator>(self, tree: &mut ViewTree, decorator: D) {
        if self == tree.root { panic!("root view decorator can not be changed"); }
        if tree.arena[self.0].decorator.replace(Box::new(decorator) as _).is_none() {
            let parent_window = self
                .self_and_parents(tree)
                .find_map(|view| tree.arena[view.0].window)
            ;
            let render_bounds = self.render_bounds(tree);
            let window = Window::new(
                tree.window_tree(),
                parent_window,
                render_bounds,
                |window| (self, window)
            );
            tree.arena[self.0].window = Some(window);
            if let Some(last_child) = self.last_child(tree) {
                let mut child = last_child;
                loop {
                    child = child.next(tree);
                    child.renew_window(tree, Some(window));
                    if child == last_child { break; }
                }
            }
        }
        self.invalidate_measure(tree);
    }

    pub fn unset_layout(self, tree: &mut ViewTree) {
        if tree.arena[self.0].layout.take().is_some() {
            self.parent(tree).map(|parent| parent.invalidate_measure(tree));
        }
    }

    pub fn set_layout<L: Layout>(self, tree: &mut ViewTree, layout: L) {
        if self == tree.root { panic!("root view layout can not be changed"); }
        tree.arena[self.0].layout = Some(Box::new(layout) as _);
        self.parent(tree).map(|parent| parent.invalidate_measure(tree));
    }

    pub fn unset_panel(self, tree: &mut ViewTree) {
        if tree.arena[self.0].panel.take().is_some() {
            self.invalidate_measure(tree);
        }
    }

    pub fn set_panel<P: Panel>(self, tree: &mut ViewTree, panel: P) {
        tree.arena[self.0].panel = Some(Box::new(panel) as _);
        self.invalidate_measure(tree);
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
            let item = view.map(|view| view.next(tree));
            view = if item == last_child { None } else { item };
            item
        })
    }

    pub fn desired_size(self, tree: &ViewTree) -> Vector { tree.arena[self.0].desired_size }

    pub fn render_bounds(self, tree: &ViewTree) -> Rect { tree.arena[self.0].render_bounds }

    pub fn decorator_get<D: Decorator + DepObj<Id=View>, T>(
        self,
        tree: &ViewTree,
        prop: DepProp<D, T>,
    ) -> &T {
        let decorator = tree.arena[self.0]
            .decorator
            .as_ref()
            .expect("Decorator missed")
            .downcast_ref::<D>()
            .expect("invalid cast")
        ;
        prop.get(decorator)
    }

    pub fn decorator_set_uncond<D: Decorator + DepObj<Id=View>, T>(
        self,
        context: &mut dyn Context,
        prop: DepProp<D, T>,
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
        let (old, on_changed) = prop.set_uncond(decorator, value);
        on_changed.raise(self, context, &old);
        old
    }

    pub fn decorator_set_distinct<D: Decorator + DepObj<Id=View>, T: Eq>(
        self,
        context: &mut dyn Context,
        prop: DepProp<D, T>,
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
        let (old, on_changed) = prop.set_distinct(decorator, value);
        on_changed.raise(self, context, &old);
        old
    }

    pub fn decorator_on_changed<D: Decorator + DepObj<Id=View>, T>(
        self,
        tree: &mut ViewTree,
        prop: DepProp<D,T>,
        on_changed: fn(owner: View, context: &mut dyn Context, old: &T),
    ) {
        let decorator = tree.arena[self.0]
            .decorator
            .as_mut()
            .expect("Decorator missed")
            .downcast_mut::<D>()
            .expect("invalid cast")
        ;
        prop.on_changed(decorator, on_changed);
    }

    pub fn layout_get<L: Layout + DepObj<Id=View>, T>(
        self,
        tree: &ViewTree,
        prop: DepProp<L, T>,
    ) -> &T {
        let layout = tree.arena[self.0]
            .layout
            .as_ref()
            .expect("Layout missed")
            .downcast_ref::<L>()
            .expect("invalid cast")
        ;
        prop.get(layout)
    }

    pub fn layout_set_uncond<L: Layout + DepObj<Id=View>, T>(
        self,
        context: &mut dyn Context,
        prop: DepProp<L, T>,
        value: T,
    ) -> T {
        let tree = context.get_mut::<ViewTree>().expect("ViewTree required");
        let layout = tree.arena[self.0]
            .layout
            .as_mut()
            .expect("Layout missed")
            .downcast_mut::<L>()
            .expect("invalid cast")
        ;
        let (old, on_changed) = prop.set_uncond(layout, value);
        on_changed.raise(self, context, &old);
        old
    }

    pub fn layout_set_distinct<L: Layout + DepObj<Id=View>, T: Eq>(
        self,
        context: &mut dyn Context,
        prop: DepProp<L, T>,
        value: T,
    ) -> T {
        let tree = context.get_mut::<ViewTree>().expect("ViewTree required");
        let layout = tree.arena[self.0]
            .layout
            .as_mut()
            .expect("Layout missed")
            .downcast_mut::<L>()
            .expect("invalid cast")
        ;
        let (old, on_changed) = prop.set_distinct(layout, value);
        on_changed.raise(self, context, &old);
        old
    }

    pub fn layout_on_changed<L: Layout + DepObj<Id=View>, T>(
        self,
        tree: &mut ViewTree,
        prop: DepProp<L, T>,
        on_changed: fn(owner: View, context: &mut dyn Context, old: &T),
    ) {
        let layout = tree.arena[self.0]
            .layout
            .as_mut()
            .expect("Layout missed")
            .downcast_mut::<L>()
            .expect("invalid cast")
        ;
        prop.on_changed(layout, on_changed);
    }

    pub fn panel_get<P: Panel + DepObj<Id=View>, T>(
        self,
        tree: &ViewTree,
        prop: DepProp<P, T>,
    ) -> &T {
        let panel = tree.arena[self.0]
            .panel
            .as_ref()
            .expect("Panel missed")
            .downcast_ref::<P>()
            .expect("invalid cast")
        ;
        prop.get(panel)
    }

    pub fn panel_set_uncond<P: Panel + DepObj<Id=View>, T>(
        self,
        context: &mut dyn Context,
        prop: DepProp<P, T>,
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
        let (old, on_changed) = prop.set_uncond(panel, value);
        on_changed.raise(self, context, &old);
        old
    }

    pub fn panel_set_distinct<P: Panel + DepObj<Id=View>, T: Eq>(
        self,
        context: &mut dyn Context,
        prop: DepProp<P, T>,
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
        let (old, on_changed) = prop.set_distinct(panel, value);
        on_changed.raise(self, context, &old);
        old
    }

    pub fn panel_on_changed<P: Panel + DepObj<Id=View>, T>(
        self,
        tree: &mut ViewTree,
        prop: DepProp<P, T>,
        on_changed: fn(owner: View, context: &mut dyn Context, old: &T),
    ) {
        let panel = tree.arena[self.0]
            .panel
            .as_mut()
            .expect("Panel missed")
            .downcast_mut::<P>()
            .expect("invalid cast")
        ;
        prop.on_changed(panel, on_changed);
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
                    child.measure(tree, children_measure_size);
                    children_desired_size = children_desired_size.max(child.desired_size(tree));
                    if child == last_child { break children_desired_size; }
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
                    child.arrange(tree, children_arrange_bounds);
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
        let render_bounds = decorator.as_ref().map_or(
            children_render_bounds,
            |d| d.render_bounds(self, tree, children_render_bounds)
        );
        let render_bounds = Rect {
            tl: rect.tl.offset(render_bounds.tl.offset_from(Point { x: 0, y: 0 })),
            size: render_bounds.size
        }.intersect(rect);
        let window = tree.arena[self.0].window;
        window.map(|w| w.move_(tree.window_tree(), render_bounds));
        tree.arena[self.0].render_bounds = render_bounds;
    }
}

dep_obj! {
    #[derive(Debug)]
    pub struct RootDecorator as View: RootDecoratorType {
        bg: Text = Text::SPACE.clone(),
    }
}

static ROOT_DECORATOR_TOKEN: sync::Lazy<DepTypeToken<RootDecoratorType>> = sync::Lazy::new(||
    RootDecoratorType::new_raw().expect("RootDecoratorType builder locked")
);

pub fn root_decorator_type() -> &'static RootDecoratorType { ROOT_DECORATOR_TOKEN.type_() }

impl RootDecorator {
    fn invalidate_bg(_view: View, context: &mut dyn Context, _old: &Text) {
        let tree = context.get_mut::<ViewTree>().expect("ViewTree required");
        tree.window_tree().invalidate_screen();
    }
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
        let bg = view.decorator_get(tree, root_decorator_type().bg());
        port.fill(|port, p| port.out(p, bg.fg, bg.bg, bg.attr, &bg.value));
    }
}

dep_obj! {
    #[derive(Debug)]
    pub struct ViewBase as View: ViewBaseType {
        min_size: Vector,
        max_size: Vector,
        size: Vector,
    }
}
