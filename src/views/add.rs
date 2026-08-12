use crate::AddMethod;
use crate::AppState;
use crate::Screen;
use crate::decode;
use crate::model::{Code, Symbology};
use crate::views::PALETTE;
use leptos::html::{Input, Video};
use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;

#[component]
pub fn Add(initial_method: AddMethod) -> impl IntoView {
    let state = use_context::<AppState>().expect("AppState");
    let method = RwSignal::new(initial_method);
    let error = RwSignal::new(None::<String>);
    let pending = RwSignal::new(false);

    let name = RwSignal::new(String::new());
    let color = RwSignal::new(PALETTE[0].to_string());
    let value = RwSignal::new(String::new());
    let symbology = RwSignal::new(Symbology::Code128);

    let back = move |_| state.screen.set(Screen::Overview);

    // ---- handling a decoded value ----
    let on_decoded = Callback::new({
        let state = state.clone();
        let error = error.clone();
        let value = value.clone();
        let symbology = symbology.clone();
        let method = method.clone();
        move |decoded: decode::Decoded| {
            let probe = Code {
                value: decoded.value.clone(),
                symbology: decoded.symbology,
                name: String::new(),
                color: String::new(),
            };
            match probe.validate() {
                Ok(()) => {
                    value.set(decoded.value.clone());
                    symbology.set(decoded.symbology);
                    error.set(None);
                    if let Some(existing) = state.find(decoded.symbology, &decoded.value) {
                        state
                            .screen
                            .set(Screen::Edit((existing.symbology, existing.value)));
                    } else {
                        method.set(AddMethod::Manual);
                    }
                }
                Err(e) => error.set(Some(e)),
            }
        }
    });

    // ---- save ----
    let save = {
        let state = state.clone();
        let error = error.clone();
        let pending = pending.clone();
        move |_| {
            let n = name.get();
            let c = color.get();
            let v = value.get().trim().to_string();
            let s = symbology.get();
            let code = Code {
                value: v,
                symbology: s,
                name: n,
                color: c,
            };
            if let Err(e) = code.validate() {
                error.set(Some(e));
                return;
            }
            if state.find(s, &code.value).is_some() {
                error.set(Some("a code with this value already exists".to_string()));
                return;
            }
            let st = state.clone();
            let err = error.clone();
            let pend = pending.clone();
            spawn_local(async move {
                pend.set(true);
                match st.store.put(&code).await {
                    Ok(()) => {
                        st.reload().await;
                        st.screen.set(Screen::Overview);
                    }
                    Err(e) => err.set(Some(e)),
                }
                pend.set(false);
            });
        }
    };

    let on_error = Callback::new(move |e: String| error.set(Some(e)));

    view! {
        <div class="screen">
            <header class="screen-header">
                <button class="back-btn" on:click=back>"‹"</button>
                <h1>"Add code"</h1>
            </header>

            <Show when=move || error.get().is_some() fallback=|| ()>
                <div class="error">{move || error.get().unwrap_or_default()}</div>
            </Show>

            <div class="method-tabs">
                <button class:active=move || method.get() == AddMethod::Camera on:click=move |_| method.set(AddMethod::Camera)>"Camera"</button>
                <button class:active=move || method.get() == AddMethod::Image on:click=move |_| method.set(AddMethod::Image)>"Image"</button>
                <button class:active=move || method.get() == AddMethod::Manual on:click=move |_| method.set(AddMethod::Manual)>"Manual"</button>
            </div>

            <div class="scan-view">
                <Show when=move || method.get() == AddMethod::Camera fallback=|| ()>
                    <CameraCapture on_decoded=on_decoded.clone() on_error=on_error.clone() />
                </Show>

                <Show when=move || method.get() == AddMethod::Image fallback=|| ()>
                    <ImageCapture on_decoded=on_decoded.clone() on_error=on_error.clone() />
                </Show>
            </div>

            <Show when=move || method.get() == AddMethod::Manual fallback=|| ()>
                <div>
                    <div class="field">
                        <label>"Symbology"</label>
                        <select
                            on:change=move |ev| {
                                let v = event_target_value(&ev);
                                symbology.set(match v.as_str() {
                                    "EAN-13" => Symbology::Ean13,
                                    "UPC-A" => Symbology::UpcA,
                                    "QR" => Symbology::QrCode,
                                    _ => Symbology::Code128,
                                });
                            }
                        >
                            <option value="Code 128" selected=move || symbology.get() == Symbology::Code128>"Code 128"</option>
                            <option value="EAN-13" selected=move || symbology.get() == Symbology::Ean13>"EAN-13"</option>
                            <option value="UPC-A" selected=move || symbology.get() == Symbology::UpcA>"UPC-A"</option>
                            <option value="QR" selected=move || symbology.get() == Symbology::QrCode>"QR"</option>
                        </select>
                    </div>

                    <div class="field">
                        <label>"Code value"</label>
                        <input prop:value=move || value.get() on:input=move |ev| value.set(event_target_value(&ev)) placeholder="e.g. 5021886…" />
                    </div>

                    <div class="field">
                        <label>"Name"</label>
                        <input prop:value=move || name.get() on:input=move |ev| name.set(event_target_value(&ev)) placeholder="e.g. Nero Caf&#233;" />
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

                    <button class="btn" disabled=move || pending.get() on:click={let save = save.clone(); move |_| save(())}>
                        {move || if pending.get() { "Saving…" } else { "Save" }}
                    </button>
                </div>
            </Show>
        </div>
    }
}

#[component]
fn CameraCapture(
    on_decoded: Callback<decode::Decoded>,
    on_error: Callback<String>,
) -> impl IntoView {
    let video_ref = NodeRef::<Video>::new();
    let started = RwSignal::new(false);
    let scanning = RwSignal::new(true);
    let on_decoded = on_decoded;
    let on_error = on_error;

    let start = move |_| {
        started.set(true);
        scanning.set(true);
    };

    // Begin camera + polling once the video element is present and user started.
    let video_ref = video_ref.clone();
    video_ref.on_load({
        let started = started.clone();
        let scanning = scanning.clone();
        let on_decoded = on_decoded.clone();
        let on_error = on_error.clone();
        move |video: web_sys::HtmlVideoElement| {
            let started2 = started.clone();
            let scanning2 = scanning.clone();
            let on_decoded2 = on_decoded.clone();
            let on_error2 = on_error.clone();
            spawn_local(async move {
                // Wait until the user presses Start.
                while !started2.get() {
                    gloo_timers::future::TimeoutFuture::new(200).await;
                }
                let window = web_sys::window().expect("window");
                let nav = window.navigator();
                let devices = nav.media_devices().map_err(|e| format!("{e:?}"));
                let stream = match devices {
                    Ok(dev) => {
                        let c = web_sys::MediaStreamConstraints::new();
                        let video = web_sys::MediaTrackConstraints::new();
                        video.set_facing_mode_str("environment");
                        c.set_video_media_track_constraints(&video);
                        c.set_audio(&wasm_bindgen::JsValue::from_bool(false));
                        let promise = dev
                            .get_user_media_with_constraints(&c)
                            .map_err(|e| format!("request failed: {e:?}"));
                        match promise {
                            Ok(p) => wasm_bindgen_futures::JsFuture::from(p)
                                .await
                                .map_err(|e| format!("denied: {e:?}"))
                                .map(|v| v.dyn_into::<web_sys::MediaStream>().expect("stream")),
                            Err(e) => Err(e),
                        }
                    }
                    Err(e) => Err(e),
                };
                let stream = match stream {
                    Ok(s) => s,
                    Err(e) => {
                        on_error.run(format!("could not access camera: {e}"));
                        return;
                    }
                };
                video.set_src_object(Some(&stream));
                let _ = video.play();

                // Poll frames and decode.
                let decoding = RwSignal::new(false);
                let interval_closure = {
                    let scanning3 = scanning2.clone();
                    let decoding3 = decoding.clone();
                    let on_decoded3 = on_decoded2.clone();
                    let on_error3 = on_error2.clone();
                    Closure::wrap(Box::new(move || {
                        if !scanning3.get() || decoding3.get() {
                            return;
                        }
                        decoding3.set(true);
                        spawn_local({
                            let scanning4 = scanning3.clone();
                            let decoding4 = decoding3.clone();
                            let on_decoded4 = on_decoded3.clone();
                            let on_error4 = on_error3.clone();
                            let video4 = video.clone();
                            async move {
                                let result = decode_video_frame(&video4);
                                match result {
                                    Ok(decoded) => {
                                        decoding4.set(false);
                                        scanning4.set(false);
                                        on_decoded4.run(decoded);
                                    }
                                    Err(_) => {
                                        decoding4.set(false);
                                        let _ = on_error4;
                                    }
                                }
                            }
                        });
                    }) as Box<dyn FnMut()>)
                };
                let _ = web_sys::window()
                    .expect("window")
                    .set_interval_with_callback_and_timeout_and_arguments_0(
                        interval_closure.as_ref().unchecked_ref(),
                        400,
                    )
                    .expect("set_interval");
            });
        }
    });

    view! {
        <div>
            <video node_ref=video_ref autoplay=true playsinline=true muted=true></video>
            <div>
                <button class="btn" on:click=start>{move || if started.get() { "Scanning…" } else { "Start camera" }}</button>
            </div>
        </div>
    }
}

fn decode_video_frame(video: &web_sys::HtmlVideoElement) -> Result<decode::Decoded, String> {
    let document = web_sys::window()
        .ok_or("no window")?
        .document()
        .ok_or("no document")?;
    let canvas: web_sys::HtmlCanvasElement = document
        .create_element("canvas")
        .map_err(|_| "no canvas")?
        .dyn_into()
        .map_err(|_| "no canvas")?;
    canvas.set_width(video.video_width());
    canvas.set_height(video.video_height());
    let ctx = canvas
        .get_context("2d")
        .map_err(|_| "no ctx")?
        .ok_or_else(|| "no ctx")?
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .map_err(|_| "no ctx")?;
    ctx.draw_image_with_html_video_element(video, 0.0, 0.0)
        .map_err(|_| "draw failed")?;
    let data_url = canvas.to_data_url().map_err(|_| "toDataURL failed")?;
    let bytes = data_url_to_bytes(&data_url);
    if bytes.is_empty() {
        return Err("empty image".into());
    }
    decode::decode_from_bytes(&bytes)
}

fn data_url_to_bytes(url: &str) -> Vec<u8> {
    // data:image/png;base64,....
    let Some(comma) = url.find(',') else {
        return Vec::new();
    };
    let b64 = &url[comma + 1..];
    let Some(window) = web_sys::window() else {
        return Vec::new();
    };
    window
        .atob(b64)
        .ok()
        .map(|decoded| decoded.bytes().collect())
        .unwrap_or_default()
}

#[component]
fn ImageCapture(
    on_decoded: Callback<decode::Decoded>,
    on_error: Callback<String>,
) -> impl IntoView {
    let input_ref = NodeRef::<Input>::new();
    let chosen = RwSignal::new(false);

    let on_pick = {
        let input_ref = input_ref.clone();
        let on_decoded = on_decoded.clone();
        let on_error = on_error.clone();
        let chosen = chosen.clone();
        move |_| {
            if let Some(input) = input_ref.get() {
                if let Some(file) = input.files().and_then(|fl| fl.get(0)) {
                    chosen.set(true);
                    let on_decoded = on_decoded.clone();
                    let on_error = on_error.clone();
                    let chosen = chosen.clone();
                    spawn_local(async move {
                        // Read the file's actual bytes. Uint8Array::new(&file) would produce
                        // an empty array (the File is not interpreted as array data), so we
                        // must await array_buffer() and wrap that ArrayBuffer instead.
                        let promise = file.array_buffer();
                        let bytes = match wasm_bindgen_futures::JsFuture::from(promise).await {
                            Ok(buf) => js_sys::Uint8Array::new(&buf).to_vec(),
                            Err(_) => {
                                on_error.run("could not read the selected file".to_string());
                                return;
                            }
                        };
                        match decode::decode_from_bytes(&bytes) {
                            Ok(d) => {
                                on_decoded.run(d);
                                chosen.set(false);
                            }
                            Err(e) => on_error.run(e),
                        }
                    });
                }
            }
        }
    };

    view! {
        <div>
            <input type="file" accept="image/*" node_ref=input_ref on:change=on_pick />
            <div>{move || if chosen.get() { "Decoding…" } else { "Choose a photo of the barcode." }}</div>
        </div>
    }
}
