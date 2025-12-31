use std::fmt::Display;

use dioxus::prelude::*;

#[derive(Clone, Copy)]
pub struct FormField<T> {
    pub value: Signal<T>,
    pub raw_value: Signal<String>,
    pub error: Signal<Option<String>>,
    pub oninput: EventHandler<FormEvent>,
    pub mounted: Signal<Option<MountedEvent>>,
}
impl<T> FormField<T> {
    pub async fn focus(&self) {
        if let Some(mounted) = &*self.mounted.read() {
            _ = mounted.set_focus(true).await;
        }
    }
}

pub fn use_form_field<T, E>(
    initial_value: T,
    validator: impl Fn(String) -> Result<T, E> + 'static,
) -> FormField<T>
where
    T: PartialEq + ToString + Clone + 'static,
    E: Display,
{
    let mut value = use_signal(|| initial_value.clone());
    let mut raw_value = use_signal(|| initial_value.to_string());
    let mut error = use_signal(|| None::<String>);
    let mounted = use_signal(|| None::<MountedEvent>);

    let callback = move |e: FormEvent| {
        let input = e.value();

        raw_value.set(input.clone());

        match validator(input) {
            Ok(v) => {
                if v != *value.read() {
                    value.set(v);
                }
                if error.read().is_some() {
                    error.set(None);
                }
            }
            Err(e) => {
                error.set(Some(e.to_string()));
            }
        };
    };

    FormField {
        value,
        raw_value,
        error,
        oninput: EventHandler::new(callback),
        mounted,
    }
}
