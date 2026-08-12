use crate::model::Symbology;
use crate::views::PALETTE;
use crate::AppState;
use crate::Screen;
use leptos::prelude::*;
use leptos::task::spawn_local;

#[component]
pub fn Edit(symbology: Symbology, value: String) -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState");
    let error = RwSignal::new(None::<String>);

    let original = state.find(symbology, &value).expect("edit target");
    let name = RwSignal::new(original.name.clone());
    let color = RwSignal::new(original.color.clone());
    let confirm = RwSignal::new(false);

    let back = move |_| state.screen.set(Screen::Overview);

    let save = {
        let state = state.clone();
        let error = error.clone();
        move |_| {
            let mut code = original.clone();
            code.name = name.get();
            code.color = color.get();
            let st = state.clone();
            let err = error.clone();
            spawn_local(async move {
                match st.store.put(&code).await {
                    Ok(()) => {
                        st.reload().await;
                        st.screen.set(Screen::Overview);
                    }
                    Err(e) => err.set(Some(e)),
                }
            });
        }
    };

    let do_delete = {
        let state = state.clone();
        let v = value.clone();
        move |_| {
            let st = state.clone();
            let s = symbology;
            let v = v.clone();
            spawn_local(async move {
                st.store.delete(s, &v).await.ok();
                st.reload().await;
                st.screen.set(Screen::Overview);
            });
        }
    };

    view! {
        <div class="screen">
            <header class="screen-header">
                <button class="back-btn" on:click=back>"‹"</button>
                <h1>"Edit code"</h1>
            </header>

            <Show when=move || error.get().is_some() fallback=|| ()>
                <div class="error">{move || error.get().unwrap_or_default()}</div>
            </Show>

            <div class="field">
                <label>"Name"</label>
                <input prop:value=name on:input=move |ev| name.set(event_target_value(&ev)) />
            </div>

            <div class="field">
                <label>"Color"</label>
                <div class="color-swatches">
                    <For each=move || PALETTE.to_vec() key=|c| c.to_string() let:sw>
                        <button
                            class:selected=move || color.get() == sw
                            class="swatch"
                            style:background=sw
                            on:click=move |_| color.set(sw.to_string())
                        ></button>
                    </For>
                </div>
            </div>

            <div class="field">
                <label>"Value (immutable)"</label>
                <input value=value.clone() disabled=true />
            </div>

            <button class="btn" on:click=save>"Save"</button>

            <Show when=move || !confirm.get() fallback=|| ()>
                <button class="btn danger" on:click=move |_| confirm.set(true)>"Delete code"</button>
            </Show>

            <Show when=move || confirm.get() fallback=|| ()>
                <div class="overlay">
                    <div class="overlay-card">
                        <h2>"Delete this code?"</h2>
                        <p>"This cannot be undone."</p>
                        <div class="overlay-actions">
                            <button class="btn ghost" on:click=move |_| confirm.set(false)>"Cancel"</button>
                            <button class="btn danger" on:click={let do_delete = do_delete.clone(); move |_| do_delete(())}>"Delete"</button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
