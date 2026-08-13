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
            let n = name.get_untracked();
            let c = color.get_untracked();
            let v = value.get_untracked().trim().to_string();
            let s = symbology.get_untracked();
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
    let preview_ref = NodeRef::<leptos::html::Canvas>::new();
    let started = RwSignal::new(false);
    let scanning = RwSignal::new(true);
    let stream_holder = RwSignal::new(None::<web_sys::MediaStream>);
    // Set when the component is torn down. Every loop and every waiting task
    // checks this so nothing keeps running after the view is gone.
    let cancelled = RwSignal::new(false);

    let start = move |_| {
        started.set(true);
        scanning.set(true);
    };

    // Stop scanning and free the camera: halt decoding and stop every track of
    // the acquired stream so the camera is released.
    let release = move || {
        scanning.set(false);
        started.set(false);
        if let Some(stream) = stream_holder.get_untracked() {
            for track in stream.get_tracks().iter() {
                if let Ok(track) = track.dyn_into::<web_sys::MediaStreamTrack>() {
                    track.stop();
                }
            }
        }
        stream_holder.set(None);
        if let Some(video) = video_ref.get_untracked() {
            video.set_src_object(None);
        }
    };

    let stop = move |_| release();

    // Leaving the screen (back button, tab switch) unmounts this component
    // without going through the Stop button, so release the camera here too and
    // flag the frame loop to exit at its next tick.
    on_cleanup(move || {
        cancelled.set(true);
        release();
    });

    // Begin camera + polling once the video element is present and user started.
    video_ref.on_load(move |video: web_sys::HtmlVideoElement| {
        spawn_local(async move {
            // Wait until the user presses Start, giving up if the view is gone.
            while !started.get_untracked() {
                if cancelled.get_untracked() {
                    return;
                }
                gloo_timers::future::TimeoutFuture::new(200).await;
            }
            let window = web_sys::window().expect("window");
            let nav = window.navigator();
            let devices = nav.media_devices().map_err(|e| format!("{e:?}"));
            let stream = match devices {
                Ok(dev) => {
                    let c = web_sys::MediaStreamConstraints::new();
                    let track = web_sys::MediaTrackConstraints::new();
                    track.set_facing_mode_str("environment");
                    // Ask for a small frame. These are bare values, so they are
                    // treated as "ideal" and a device that cannot deliver them
                    // still returns its closest match rather than failing.
                    // A barcode only needs enough pixels to resolve the
                    // narrowest bar, and every extra pixel is decode work.
                    track.set_width_i32(CAPTURE_WIDTH);
                    track.set_height_i32(CAPTURE_HEIGHT);
                    track.set_frame_rate_f64(CAPTURE_FPS);
                    c.set_video_media_track_constraints(&track);
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
            // The view may have been torn down while permission was pending;
            // in that case hand the camera straight back.
            if cancelled.get_untracked() {
                for track in stream.get_tracks().iter() {
                    if let Ok(track) = track.dyn_into::<web_sys::MediaStreamTrack>() {
                        track.stop();
                    }
                }
                return;
            }
            stream_holder.set(Some(stream.clone()));
            video.set_src_object(Some(&stream));
            let _ = video.play();

            // Wait for the video to actually hold a frame. readyState is the
            // documented signal for this; non-zero dimensions can be reported
            // before any pixels are available, which yields blank captures.
            for _ in 0..50 {
                if cancelled.get_untracked() {
                    return;
                }
                if video.ready_state() >= web_sys::HtmlMediaElement::HAVE_CURRENT_DATA {
                    break;
                }
                gloo_timers::future::TimeoutFuture::new(100).await;
            }
            if video.ready_state() < web_sys::HtmlMediaElement::HAVE_CURRENT_DATA {
                on_error.run("camera did not start".to_string());
                return;
            }

            let Some(preview) = preview_ref.get_untracked() else {
                on_error.run("camera did not start".to_string());
                return;
            };

            // Frame loop. Each pass runs to completion and only then schedules
            // the next one, so a slow decode can never queue up behind itself
            // and the loop self-throttles to whatever the device can manage.
            loop {
                if cancelled.get_untracked() || !scanning.get_untracked() {
                    return;
                }
                if video.ready_state() >= web_sys::HtmlMediaElement::HAVE_CURRENT_DATA {
                    match decode_video_frame(&video, &preview) {
                        Ok(decoded) => {
                            scanning.set(false);
                            on_decoded.run(decoded);
                            return;
                        }
                        Err(e) => {
                            // "no barcode found" is the normal state between
                            // successful frames; log instead of surfacing to UI.
                            log::debug!("scan frame: {e}");
                        }
                    }
                }
                // Yield to the browser so it can composite the live preview and
                // process input before the next decode occupies the main thread.
                gloo_timers::future::TimeoutFuture::new(FRAME_GAP_MS).await;
            }
        });
    });

    view! {
        <div>
            // The video is shown directly, so the preview is composited by the
            // browser at full frame rate and stays smooth no matter how long a
            // decode takes. The canvas is only a scratch buffer for capture.
            <video node_ref=video_ref autoplay=true playsinline=true muted=true class="camera-preview"></video>
            <canvas node_ref=preview_ref class="camera-scratch"></canvas>
            <div>
                <Show when=move || !started.get() fallback=|| ()>
                    <button class="btn" on:click=start>"Start camera"</button>
                </Show>
                <Show when=move || started.get() fallback=|| ()>
                    <button class="btn secondary" on:click=stop>"Stop scanning"</button>
                </Show>
            </div>
        </div>
    }
}

/// Frame size requested from the camera.
const CAPTURE_WIDTH: i32 = 640;
const CAPTURE_HEIGHT: i32 = 480;
const CAPTURE_FPS: f64 = 15.0;

/// Pause between the end of one decode and the start of the next.
const FRAME_GAP_MS: u32 = 60;

fn decode_video_frame(
    video: &web_sys::HtmlVideoElement,
    preview: &web_sys::HtmlCanvasElement,
) -> Result<decode::Decoded, String> {
    let width = video.video_width();
    let height = video.video_height();
    if width == 0 || height == 0 {
        return Err("video not ready".into());
    }

    // Draw at the size the decoder works at, so it never has to rescale.
    let longest = width.max(height);
    let (draw_w, draw_h) = if longest > decode::MAX_SIDE {
        let scale = decode::MAX_SIDE as f64 / longest as f64;
        (
            (width as f64 * scale).round().max(1.0) as u32,
            (height as f64 * scale).round().max(1.0) as u32,
        )
    } else {
        (width, height)
    };

    if preview.width() != draw_w {
        preview.set_width(draw_w);
    }
    if preview.height() != draw_h {
        preview.set_height(draw_h);
    }

    // willReadFrequently keeps the canvas backing store on the CPU. Without it
    // the per-frame getImageData below forces a GPU readback every time.
    let options = js_sys::Object::new();
    js_sys::Reflect::set(
        &options,
        &wasm_bindgen::JsValue::from_str("willReadFrequently"),
        &wasm_bindgen::JsValue::TRUE,
    )
    .map_err(|_| "no ctx")?;
    let ctx = preview
        .get_context_with_context_options("2d", &options)
        .map_err(|_| "no ctx")?
        .ok_or("no ctx")?
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .map_err(|_| "no ctx")?;
    ctx.draw_image_with_html_video_element_and_dw_and_dh(
        video, 0.0, 0.0, draw_w as f64, draw_h as f64,
    )
    .map_err(|_| "draw failed")?;
    let image_data = ctx
        .get_image_data(0.0, 0.0, draw_w as f64, draw_h as f64)
        .map_err(|_| "getImageData failed")?;
    let rgba: Vec<u8> = image_data.data().0;
    decode::decode_from_rgba(draw_w, draw_h, &rgba)
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
