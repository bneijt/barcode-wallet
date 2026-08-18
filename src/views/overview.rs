use crate::import_export;
use crate::model::{ordinal_word, Code};
use crate::AppState;
use crate::Screen;
use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsCast;

#[component]
pub fn Overview() -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState");
    let open_chooser = RwSignal::new(false);
    let open_menu = RwSignal::new(false);

    view! {
        <div class="overview">
            <header class="overview-header">
                <h1>"Barcode Wallet"</h1>
                <button class="menu-btn" aria-label="Menu" on:click=move |_| open_menu.set(true)>"☰"</button>
            </header>

            <Show
                when=move || { !state.loading.get() && state.codes.get().is_empty() }
                fallback=|| ()
            >
                <div class="empty">"No codes yet. Tap + to add your first one."</div>
            </Show>

            <div class="grid">
                <For
                    each=move || state.codes.get()
                    key=|c| (c.symbology, c.value.clone())
                    let:code
                >
                    <Tile code=code />
                </For>
            </div>

            <button class="fab" on:click=move |_| open_chooser.set(true)>"+"</button>

            <Show when=move || open_chooser.get() fallback=|| ()>
                <Chooser on_close=Callback::new(move |_| open_chooser.set(false)) />
            </Show>

            <Show when=move || open_menu.get() fallback=|| ()>
                <Menu state=state.clone() on_close=Callback::new(move |_| open_menu.set(false)) />
            </Show>
        </div>
    }
}

#[component]
fn Tile(code: Code) -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState");
    let on_click = {
        let state = state.clone();
        let id = (code.symbology, code.value.clone());
        move |_| state.screen.set(Screen::Display(id.clone()))
    };

    let ordinal = state
        .codes
        .get_untracked()
        .into_iter()
        .filter(|c| c.name == code.name)
        .position(|c| c.value == code.value && c.symbology == code.symbology);
    let ordinal_text = ordinal.map(|n| ordinal_word(n + 1)).unwrap_or_default();

    view! {
        <button class="tile" style:background=code.color.clone() on:click=on_click>
            <span class="tile-name">{code.name.clone()}</span>
            <span class="tile-footer">
                <span class="tile-ordinal">{ordinal_text}</span>
                <span class="tile-symbology">{code.symbology.display_name()}</span>
            </span>
        </button>
    }
}

#[component]
fn Chooser(on_close: Callback<()>) -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState");
    let on_close = move || on_close.run(());

    view! {
        <div class="chooser" on:click=move |_| on_close()>
            <div class="chooser-sheet" on:click=move |ev| ev.stop_propagation()>
                <button on:click={let st = state.clone(); move |_| st.screen.set(Screen::Add(crate::AddMethod::Camera))}>
                    "Capture with camera"
                </button>
                <button on:click={let st = state.clone(); move |_| st.screen.set(Screen::Add(crate::AddMethod::Image))}>
                    "Select an image"
                </button>
                <button on:click={let st = state.clone(); move |_| st.screen.set(Screen::Add(crate::AddMethod::Manual))}>
                    "Enter code manually"
                </button>
                <button class="chooser-cancel" on:click=move |_| on_close()>"Cancel"</button>
            </div>
        </div>
    }
}

#[component]
fn Menu(state: AppState, on_close: Callback<()>) -> impl IntoView {
    let import_ref = NodeRef::<leptos::html::Input>::new();
    let summary = RwSignal::new(None::<crate::model::ImportSummary>);
    let error = RwSignal::new(None::<String>);
    let notice = RwSignal::new(None::<String>);
    let close = move || on_close.run(());

    // Export: serialize the current collection and trigger a download.
    let do_export = {
        let state = state.clone();
        move || {
            let codes = state.codes.get_untracked();
            let json = import_export::export_to_json(&codes);
            download_json(&json, &export_filename());
        }
    };

    // Import: read a picked file, parse, and store new Codes.
    let do_import = {
        let state = state.clone();
        let summary = summary.clone();
        let error = error.clone();
        move || {
            if let Some(input) = import_ref.get() {
                if let Some(file) = input.files().and_then(|fl| fl.get(0)) {
                    let st = state.clone();
                    let s = summary.clone();
                    let e = error.clone();
                    spawn_local(async move {
                        match read_file_text(&file).await {
                            Ok(text) => {
                                let existing = st.codes.get_untracked();
                                match import_export::import_from_json(&text, &existing) {
                                    Ok((added, sum)) => {
                                        for code in added {
                                            let _ = st.store.put(&code).await;
                                        }
                                        st.reload().await;
                                        s.set(Some(sum));
                                        e.set(None);
                                    }
                                    Err(err) => e.set(Some(err)),
                                }
                            }
                            Err(err) => e.set(Some(err)),
                        }
                    });
                }
            }
        }
    };

    let version = env!("CARGO_PKG_VERSION");

    // Share App: open the native share sheet with the app's canonical link, or
    // copy the link as a fallback when Web Share is unavailable.
    let do_share = move || {
        spawn_local(async move {
            match share_app().await {
                ShareOutcome::Shared => {}
                ShareOutcome::Copied => notice.set(Some("Link copied to clipboard".to_string())),
                ShareOutcome::Unsupported => {
                    error.set(Some("Sharing is not supported here, and linking failed".to_string()))
                }
            }
        });
    };

    view! {
        <div class="menu-overlay" on:click=move |_| close()>
            <div class="menu-sheet" on:click=move |ev| ev.stop_propagation()>
                <button class="menu-item" on:click=move |_| {
                    // Deliberately keep the menu open: the file input lives
                    // inside it, and the import result message is shown there.
                    // Closing first detaches the input, which can silently
                    // swallow the `change` event on some mobile browsers.
                    if let Some(input) = import_ref.get() {
                        let _ = input.click();
                    }
                }>
                    "Import"
                </button>
                <button class="menu-item" on:click={let close = close.clone(); move |_| { close(); do_export(); }}>
                    "Export"
                </button>
                <button class="menu-item" on:click=move |_| do_share()>
                    "Share app"
                </button>
                <button class="menu-item" on:click={let st = state.clone(); move |_| st.screen.set(Screen::About)}>
                    "About"
                </button>

                <input
                    type="file"
                    accept="application/json,.json"
                    node_ref=import_ref
                    class="hidden-input"
                    on:change=move |ev| {
                        do_import();
                        // Clear the selection so choosing the same file again
                        // fires a `change` event on the next pick.
                        if let Some(input) = ev
                            .target()
                            .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
                        {
                            input.set_value("");
                        }
                    }
                />

                <Show when=move || summary.get().is_some() fallback=|| ()>
                    <div class="menu-message">
                        {move || {
                            let s = summary.get().unwrap_or_default();
                            format!(
                                "Imported {}, skipped {}, rejected {}",
                                s.imported, s.skipped, s.rejected
                            )
                        }}
                    </div>
                </Show>

                <Show when=move || notice.get().is_some() fallback=|| ()>
                    <div class="menu-message">{move || notice.get().unwrap_or_default()}</div>
                </Show>

                <Show when=move || error.get().is_some() fallback=|| ()>
                    <div class="menu-message error">{move || error.get().unwrap_or_default()}</div>
                </Show>

                <div class="menu-version">"Version " {version}</div>
            </div>
        </div>
    }
}

/// Canonial, installable URL for the app, used when sharing.
const APP_URL: &str = "https://barcode-wallet.bneijt.nl/";

/// Result of trying to share the app link.
enum ShareOutcome {
    /// The native share sheet handled it.
    Shared,
    /// Web Share was unavailable, so the link was copied to the clipboard.
    Copied,
    /// Neither sharing nor copying could be done.
    Unsupported,
}

/// Share the app via the native share sheet, falling back to copying the link.
///
/// The Web Share API only exists in secure contexts and only wants to be called
/// from a user gesture, so both call sites here are an awaited click handler.
async fn share_app() -> ShareOutcome {
    let Some(window) = web_sys::window() else {
        return ShareOutcome::Unsupported;
    };
    let navigator = window.navigator();

    // Feature-detect navigator.share; the web-sys binding is not `catch`, so
    // calling it on a browser without the API would throw before returning a
    // Promise.
    let share_present = js_sys::Reflect::get(&navigator, &wasm_bindgen::JsValue::from_str("share"))
        .map(|v| !v.is_undefined() && !v.is_null())
        .unwrap_or(false);

    if share_present {
        let data = web_sys::ShareData::new();
        data.set_title("Barcode Wallet");
        data.set_text("Barcode Wallet – store and display your loyalty barcodes.");
        data.set_url(APP_URL);
        let promise = navigator.share_with_data(&data);
        let result = wasm_bindgen_futures::JsFuture::from(promise).await;
        match result {
            Ok(_) => return ShareOutcome::Shared,
            Err(e) => {
                // AbortError means the user dismissed the sheet; not a failure.
                if let Ok(dom) = e.dyn_into::<web_sys::DomException>() {
                    if dom.name() == "AbortError" {
                        return ShareOutcome::Shared;
                    }
                }
                // Otherwise fall through to the clipboard fallback.
            }
        }
    }

    // Fallback: copy the link to the clipboard.
    let clipboard_present =
        js_sys::Reflect::get(&navigator, &wasm_bindgen::JsValue::from_str("clipboard"))
            .map(|v| !v.is_undefined() && !v.is_null())
            .unwrap_or(false);
    if !clipboard_present {
        return ShareOutcome::Unsupported;
    }
    let promise = navigator.clipboard().write_text(APP_URL);
    match wasm_bindgen_futures::JsFuture::from(promise).await {
        Ok(_) => ShareOutcome::Copied,
        Err(_) => ShareOutcome::Unsupported,
    }
}

/// Trigger a browser download of `text` as `filename`.
fn download_json(text: &str, filename: &str) {
    let Some(window) = web_sys::window() else { return };
    let Some(document) = window.document() else { return };
    let parts = js_sys::Array::new();
    parts.push(&wasm_bindgen::JsValue::from_str(text));
    let opts = web_sys::BlobPropertyBag::new();
    opts.set_type("application/json");
    let Ok(blob) = web_sys::Blob::new_with_str_sequence_and_options(&parts, &opts) else {
        return;
    };
    let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob) else {
        return;
    };
    let Ok(a) = document.create_element("a") else {
        let _ = web_sys::Url::revoke_object_url(&url);
        return;
    };
    if let Ok(a) = a.dyn_into::<web_sys::HtmlAnchorElement>() {
        a.set_href(&url);
        a.set_download(filename);
        if let Some(body) = document.body() {
            let _ = body.append_child(a.as_ref());
        }
        let _ = a.click();
        let _ = a.remove();
    }
    let _ = web_sys::Url::revoke_object_url(&url);
}

/// Date-only filename, e.g. `barcode-wallet-2026-08-11.json`.
fn export_filename() -> String {
    let now = js_sys::Date::new_0();
    let year = now.get_full_year();
    let month = format!("{:02}", now.get_month() + 1);
    let day = format!("{:02}", now.get_date());
    format!("barcode-wallet-{year}-{month}-{day}.json")
}

/// Read a File's contents as a UTF-8 string.
async fn read_file_text(file: &web_sys::File) -> Result<String, String> {
    let promise = file.array_buffer();
    let buf = wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .map_err(|_| "could not read the selected file".to_string())?;
    let u8arr = js_sys::Uint8Array::new(&buf);
    let mut bytes = Vec::with_capacity(u8arr.length() as usize);
    bytes.extend_from_slice(&u8arr.to_vec());
    String::from_utf8(bytes).map_err(|_| "file is not valid UTF-8 text".to_string())
}
