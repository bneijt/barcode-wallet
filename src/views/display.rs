use crate::model::Symbology;
use crate::render;
use crate::AppState;
use crate::Screen;
use leptos::prelude::*;
use leptos::html::Canvas;

#[component]
pub fn Display(symbology: Symbology, value: String) -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState");
    let canvas_ref = NodeRef::<Canvas>::new();
    let error = RwSignal::new(None::<String>);

    {
        let state = state.clone();
        let canvas_ref = canvas_ref.clone();
        let error = error.clone();
        let symbology = symbology;
        let value = value.clone();
        canvas_ref.on_load(move |canvas: web_sys::HtmlCanvasElement| {
            if let Some(code) = state.find(symbology, &value) {
                match render::render(&code, &canvas) {
                    Ok(()) => error.set(None),
                    Err(e) => error.set(Some(e)),
                }
            }
        });
    }

    let back = move |_| state.screen.set(Screen::Overview);
    let edit = {
        let state = state.clone();
        let id = (symbology, value.clone());
        move |_| state.screen.set(Screen::Edit(id.clone()))
    };
    let name = state.find(symbology, &value).map(|c| c.name).unwrap_or_default();

    view! {
        <div class="display">
            <button class="display-back" on:click=back aria-label="Back">"‹"</button>
            <button class="display-edit" on:click=edit aria-label="Edit">"✎"</button>
            <canvas node_ref=canvas_ref></canvas>
            <Show when=move || error.get().is_some() fallback=|| ()>
                <div class="display-label" style="color:#d33">{move || error.get().unwrap_or_default()}</div>
            </Show>
            <pre class="display-value">{value.clone()}</pre>
            <div class="display-label">{name}</div>
            <div class="display-symbology">{symbology.display_name()}</div>
        </div>
    }
}
