pub mod decode;
pub mod import_export;
pub mod model;
pub mod render;
pub mod store;
pub mod views;

use leptos::prelude::*;
use model::Code;
use send_wrapper::SendWrapper;
use std::rc::Rc;
use store::Store;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).ok();
    register_service_worker();

    // Mount into the #app container. It is empty in index.html; if WebAssembly
    // is unavailable the Rust code below never runs and the static fallback
    // markup inside #app stays visible instead.
    if let Some(app) = leptos::prelude::document().get_element_by_id("app") {
        if let Some(el) = app.dyn_into::<web_sys::HtmlElement>().ok() {
            while let Some(child) = el.first_element_child() {
                let _ = el.remove_child(&child);
            }
            leptos::prelude::mount_to(el, App).forget();
            return;
        }
    }
    // No #app container: fall back to mounting into <body>.
    leptos::prelude::mount_to_body(App);
}

fn register_service_worker() {
    let Some(window) = web_sys::window() else { return };
    let navigator = window.navigator();

    // navigator.serviceWorker only exists in secure contexts (HTTPS or
    // localhost). Reflect::get lets us detect its absence instead of
    // dereferencing an undefined binding.
    let sw = js_sys::Reflect::get(&navigator, &wasm_bindgen::JsValue::from_str("serviceWorker"))
        .unwrap_or_else(|_| wasm_bindgen::JsValue::UNDEFINED);
    if sw.is_undefined() || sw.is_null() {
        return;
    }
    let sw: web_sys::ServiceWorkerContainer = sw.unchecked_into();
    // Version in the registration URL busts HTTP caches and forces the browser
    // to re-check the script whenever Cargo.toml's version changes.
    let promise = sw.register(&format!("/sw.js?v={}", env!("CARGO_PKG_VERSION")));
    let _ = wasm_bindgen_futures::JsFuture::from(promise);
}

/// The active screen of the app.
#[derive(Clone)]
pub enum Screen {
    Overview,
    Display((model::Symbology, String)),
    Add(AddMethod),
    Edit((model::Symbology, String)),
    About,
}

/// How a new Code is captured in the Add screen.
#[derive(Clone, Copy, PartialEq)]
pub enum AddMethod {
    Camera,
    Image,
    Manual,
}

/// App-wide shared state.
#[derive(Clone)]
pub struct AppState {
    pub store: SendWrapper<Rc<Store>>,
    /// All codes; updated after every load/mutation.
    pub codes: RwSignal<Vec<Code>>,
    /// True while the store is being loaded.
    pub loading: RwSignal<bool>,
    /// The active screen.
    pub screen: RwSignal<Screen>,
}

impl AppState {
    pub fn new(store: Store) -> Self {
        Self {
            store: SendWrapper::new(Rc::new(store)),
            codes: RwSignal::new(Vec::new()),
            loading: RwSignal::new(true),
            screen: RwSignal::new(Screen::Overview),
        }
    }

    /// Reload all codes from the store into the signal.
    pub async fn reload(&self) {
        match self.store.all().await {
            Ok(codes) => self.codes.set(codes),
            Err(e) => log::error!("load codes: {e}"),
        }
    }

    /// Find a code by its stable identity.
    pub fn find(&self, symbology: model::Symbology, value: &str) -> Option<Code> {
        self.codes
            .get_untracked()
            .into_iter()
            .find(|c| c.symbology == symbology && c.value == value)
    }
}

#[component]
pub fn App() -> impl IntoView {
    let state = RwSignal::new(None::<AppState>);

    Effect::new(move |_| {
        let target = state;
        leptos::task::spawn_local(async move {
            if let Ok(store) = Store::open().await {
                target.set(Some(AppState::new(store)));
            }
        });
    });

    view! {
        <div>
            <Show
                when=move || state.get().is_some()
                fallback=move || view! { <div class="empty">"Loading…"</div> }
            >
                {move || {
                    state.get().map(|st| {
                        provide_context(st.clone());
                        view! { <views::AppView/> }
                    })
                }}
            </Show>
        </div>
    }
}
