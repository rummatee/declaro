#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub mod hooks {
    use dioxus::prelude::*;

    pub fn use_derivation<T, F>(mut derive: F) -> Signal<T>
    where
        T: 'static,
        F: FnMut() -> T + 'static,
    {
        let mut value = use_signal(&mut derive);

        use_effect(move || {
            value.set(derive());
        });

        value
    }

    pub fn use_analysis_host() -> Signal<(ide::AnalysisHost, ide::FileId)> {
        use_context::<Signal<(ide::AnalysisHost, ide::FileId)>>()
    }
}
