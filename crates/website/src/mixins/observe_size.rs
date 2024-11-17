use std::rc::Rc;
use dominator::DomBuilder;
use futures_signals::signal::Mutable;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{Element, ResizeObserver, ResizeObserverEntry};

struct ObserverWrapper<T> {
    _listener: Closure<dyn FnMut(T)>,
    observer: ResizeObserver,
}

impl<T> Drop for ObserverWrapper<T> {
    fn drop(&mut self) {
        self.observer.disconnect();
    }
}

pub trait SizeReceiver {
    fn new_size(&self, size: (f64, f64)) -> ();
}

impl SizeReceiver for Mutable<(f64, f64)> {
    fn new_size(&self, size: (f64, f64)) -> () {
        self.set(size)
    }
}

impl<T: Fn((f64, f64)) -> ()> SizeReceiver for T {
    fn new_size(&self, size: (f64, f64)) -> () {
        self(size)
    }
}

pub fn observe_size_mixin<T: AsRef<Element> + Clone + 'static>(receiver: impl SizeReceiver  + 'static) -> impl FnOnce(DomBuilder<T>) -> DomBuilder<T> {
    move |b| {
        let listener = Closure::<dyn FnMut(_)>::new(move |entries: Vec<ResizeObserverEntry>| {
            for entry in entries {
                let rect = entry.target().get_bounding_client_rect();

                let w = rect.width();
                let h = rect.height();
                receiver.new_size((w, h))
            }
        });
        let observer = ResizeObserver::new(listener.as_ref().unchecked_ref()).unwrap();

        let observer = Rc::new(ObserverWrapper {
            _listener: listener,
            observer
        });

        with_node!(b, element => {
            .apply(clone!(observer => move |b| {
                observer.observer.observe(element.as_ref());
                b
            }))
            .after_removed(move |_| {
                drop(observer);
            })
        })
    }
}