use std::mem::replace;

pub struct Property<O, T> {
    value: T,
    signals: Vec<fn(o: &mut O, old: &T)>,
}

pub struct PropertyOnChanged<O, T> {
    signals: Vec<fn(o: &mut O, old: &T)>,
}

impl<O, T> PropertyOnChanged<O, T> {
    pub fn raise(self, o: &mut O, old: &T) {
        for signal in self.signals {
            signal(o, old);
        }
    }
}

impl<O, T> Property<O, T> {
    pub fn set(&mut self, value: T) -> (T, PropertyOnChanged<O, T>) {
        let r = replace(&mut self.value, value);
        (r, PropertyOnChanged { signals: self.signals.clone() })
    }

    pub fn get(&self) -> &T { &self.value }

    pub fn on_changed(&mut self, s: fn(o: &mut O, old: &T)) { self.signals.push(s); }
}

#[macro_export]
macro_rules! property {
    ($t:ty, $name:ident, $set_name:ident, $name_on_changed:ident) => {
        pub fn $name(&self) -> &$t { self.$name.get() }

        pub fn $name_on_changed(&mut self, s: fn(o: &mut Self, old: &$t)) {
            self.$name.on_changed(s);
        }

        pub fn $set_name(&mut self, value: $t) -> $t { 
            let (old, on_changed) = self.$name.set(value);
            on_changed.raise(self, &old);
            old
        }
    }
}
