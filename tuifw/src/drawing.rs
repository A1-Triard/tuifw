use boow::Bow;
use tuifw_property::Property;
use tuifw_property::context::{ContextMutRef};
use tuifw_window::{DrawingPort, Window, WindowTree};

pub trait Drawing {
    fn draw<Error>(&self, port: &mut DrawingPort<Error>);
}

pub type DrawingContext<WindowTag, Error> = ContextMutRef<WindowTree<WindowTag, Error>, Window<WindowTag>>;

pub struct Box<Tag, WindowTag, Error> {
    pub tag: Tag,
    tl: Property<Self, Option<Bow<'static, &'static str>>, DrawingContext<WindowTag, Error>>,
/*    tr: Property<Self, Option<Bow<'static, &'static str>>>,
    bl: Property<Self, Option<Bow<'static, &'static str>>>,
    br: Property<Self, Option<Bow<'static, &'static str>>>,
    l: Property<Self, Option<Bow<'static, &'static str>>>,
    t: Property<Self, Option<Bow<'static, &'static str>>>,
    r: Property<Self, Option<Bow<'static, &'static str>>>,
    b: Property<Self, Option<Bow<'static, &'static str>>>,*/
}

impl<Tag, WindowTag, Error> Box<Tag, WindowTag, Error> {
    fn invalidate_window<T>(&mut self, context: &mut DrawingContext<WindowTag, Error>, _old: &T) {
        let _window = &mut *context.get_1();
        let _tree = context.get_2();
    }

    pub fn new(tag: Tag) -> Self {
        let mut d = Box {
            tag,
            tl: Property::new(None),
            /*tr: Property::new(None),
            bl: Property::new(None),
            br: Property::new(None),
            l: Property::new(None),
            t: Property::new(None),
            r: Property::new(None),
            b: Property::new(None),*/
        };
        d.on_changed_tl(Self::invalidate_window::<Option<Bow<'static, &'static str>>>);
        d
    }

    property!(Option<Bow<'static, &'static str>>, tl, set_tl, on_changed_tl, DrawingContext<WindowTag, Error>);
/*    property!(Option<Bow<'static, &'static str>>, tr, set_tr, on_changed_tr);
    property!(Option<Bow<'static, &'static str>>, bl, set_bl, on_changed_bl);
    property!(Option<Bow<'static, &'static str>>, br, set_br, on_changed_br);
    property!(Option<Bow<'static, &'static str>>, l, set_l, on_changed_l);
    property!(Option<Bow<'static, &'static str>>, t, set_t, on_changed_t);
    property!(Option<Bow<'static, &'static str>>, r, set_r, on_changed_r);
    property!(Option<Bow<'static, &'static str>>, b, set_b, on_changed_b);*/
}
