use dioxus::prelude::*;

pub fn use_derivation<T, F>(mut derive: F) -> Signal<T>
where
    T: 'static,
    F: FnMut() -> T + 'static,
{
    let mut value = use_signal(|| derive());

    use_effect(move || {
        value.set(derive());
    });

    value
}
