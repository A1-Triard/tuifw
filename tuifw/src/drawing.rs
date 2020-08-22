use boow::Bow;
use tuifw_property::Property;
use tuifw_property::context::{ContextMutRef};
use tuifw_window::{DrawingPort, Window, WindowTree};

pub trait Drawing<Error> {
    fn draw(&self, tree: &WindowTree<WindowTag, Error>, window: Window<WindowTag>, port: &mut DrawingPort<Error>);
}

pub type Str = Bow<'static, &'static str>;

pub type DrawingContext<WindowTag, Error> = ContextMutRef<WindowTree<WindowTag, Error>, Window<WindowTag>>;

pub struct Border<Tag, WindowTag, Error> {
    pub tag: Tag,
    tl: Property<Self, Option<Str>, DrawingContext<WindowTag, Error>>,
    tr: Property<Self, Option<Str>, DrawingContext<WindowTag, Error>>,
    bl: Property<Self, Option<Str>, DrawingContext<WindowTag, Error>>,
    br: Property<Self, Option<Str>, DrawingContext<WindowTag, Error>>,
    l: Property<Self, Option<Str>, DrawingContext<WindowTag, Error>>,
    t: Property<Self, Option<Str>, DrawingContext<WindowTag, Error>>,
    r: Property<Self, Option<Str>, DrawingContext<WindowTag, Error>>,
    b: Property<Self, Option<Str>, DrawingContext<WindowTag, Error>>,
}

impl<Tag, WindowTag, Error> Border<Tag, WindowTag, Error> {
    fn invalidate_window<T>(&mut self, context: &mut DrawingContext<WindowTag, Error>, _old: &T) {
        let _window = &mut *context.get_1();
        let _tree = context.get_2();
    }

    pub fn new(tag: Tag) -> Self {
        let mut d = Border {
            tag,
            tl: Property::new(None),
            tr: Property::new(None),
            bl: Property::new(None),
            br: Property::new(None),
            l: Property::new(None),
            t: Property::new(None),
            r: Property::new(None),
            b: Property::new(None),
        };
        d.on_changed_tl(Self::invalidate_window::<Option<Str>>);
        d
    }

    property!(Option<Str>, tl, set_tl, on_changed_tl, DrawingContext<WindowTag, Error>);
    property!(Option<Str>, tr, set_tr, on_changed_tr, DrawingContext<WindowTag, Error>);
    property!(Option<Str>, bl, set_bl, on_changed_bl, DrawingContext<WindowTag, Error>);
    property!(Option<Str>, br, set_br, on_changed_br, DrawingContext<WindowTag, Error>);
    property!(Option<Str>, l, set_l, on_changed_l, DrawingContext<WindowTag, Error>);
    property!(Option<Str>, t, set_t, on_changed_t, DrawingContext<WindowTag, Error>);
    property!(Option<Str>, r, set_r, on_changed_r, DrawingContext<WindowTag, Error>);
    property!(Option<Str>, b, set_b, on_changed_b, DrawingContext<WindowTag, Error>);
}

impl<Tag, WindowTag, Error> Drawing<Error> for Border<Tag, WindowTag, Error> {
    fn draw(&self, tree: &WindowTree<WindowTag, Error>, window: Window<WindowTag>, port: &mut DrawingPort<Error>) {
        port.out(
    }
}
