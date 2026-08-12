mod about;
mod add;
mod display;
mod edit;
mod overview;

use crate::AppState;
use crate::Screen;
use leptos::prelude::*;

#[component]
pub fn AppView() -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState");

    {
        let state = state.clone();
        Effect::new(move |_| {
            let st = state.clone();
            leptos::task::spawn_local(async move {
                st.reload().await;
                st.loading.set(false);
            });
        });
    }

    view! {
        {move || match state.screen.get() {
            Screen::Overview => view! { <overview::Overview/> }.into_any(),
            Screen::Add(method) => view! { <add::Add initial_method=method /> }.into_any(),
            Screen::Display((sym, value)) => view! { <display::Display symbology=sym value=value /> }.into_any(),
            Screen::Edit((sym, value)) => view! { <edit::Edit symbology=sym value=value /> }.into_any(),
            Screen::About => view! { <about::About/> }.into_any(),
        }}
    }
}

/// Color palette offered to the user.
pub const PALETTE: [&str; 14] = [
    "#f5d76e", "#ff9f43", "#ff6b6b", "#ee5a24", "#f3a683", "#7bed9f", "#2ed573", "#70a1ff",
    "#a29bfe", "#dfe6e9", "#ffd93d", "#6bcb77", "#4d96ff", "#ff8fa3",
];
