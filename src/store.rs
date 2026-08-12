use crate::model::{Code, Symbology};
use js_sys::Array;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    IdbDatabase, IdbFactory, IdbObjectStore, IdbOpenDbRequest, IdbRequest, IdbTransactionMode,
    IdbVersionChangeEvent,
};

const DB_NAME: &str = "barcode-wallet";
const STORE: &str = "codes";

pub struct Store {
    db: IdbDatabase,
}

fn factory() -> Result<IdbFactory, String> {
    web_sys::window()
        .ok_or("no window")?
        .indexed_db()
        .map_err(|_| "no indexedDB".to_string())?
        .ok_or_else(|| "no indexedDB".to_string())
}

fn code_key(symbology: Symbology, value: &str) -> String {
    format!("{}:{value}", symbology_name(symbology))
}

fn symbology_name(s: Symbology) -> &'static str {
    match s {
        Symbology::Code128 => "code128",
        Symbology::Ean13 => "ean13",
        Symbology::UpcA => "upca",
        Symbology::QrCode => "qr",
    }
}

/// Wrap an IndexedDB request in a JS Promise so it can be awaited.
fn request_to_future(req: IdbRequest) -> Result<JsFuture, String> {
    let promise = js_sys::Promise::new(&mut |resolve, reject| {
        let req_ok = req.clone();
        let onsuccess = Closure::once_into_js(move || {
            let _ = resolve.call1(&JsValue::UNDEFINED, &req_ok.result().unwrap());
        });
        req.clone().set_onsuccess(Some(onsuccess.unchecked_ref()));

        let req_err = req.clone();
        let onerror = {
            let req_err = req_err.clone();
            Closure::once_into_js(move || {
                let msg = req_err
                    .error()
                    .ok()
                    .flatten()
                    .map(|e| e.name())
                    .unwrap_or_else(|| "error".to_string());
                let _ = reject.call1(&JsValue::UNDEFINED, &JsValue::from_str(&msg));
            })
        };
        req.set_onerror(Some(onerror.unchecked_ref()));
    });
    Ok(JsFuture::from(promise))
}

impl Store {
    pub async fn open() -> Result<Store, String> {
        let factory = factory()?;
        let req: IdbOpenDbRequest = factory
            .open(DB_NAME)
            .map_err(|e| format!("open request failed: {e:?}"))?
            .unchecked_into();

        // On first open (no existing db), create the object store.
        let upg_req = req.clone();
        let onupgradeneeded = Closure::once(move |_ev: IdbVersionChangeEvent| {
            let db: IdbDatabase = upg_req.result().unwrap().dyn_into().expect("db");
            let _ = db.create_object_store(STORE);
        });
        req.set_onupgradeneeded(Some(onupgradeneeded.as_ref().unchecked_ref()));

        let promise = {
            let req = req.clone();
            js_sys::Promise::new(&mut |resolve, reject| {
                let req_ok = req.clone();
                let onsuccess = Closure::once_into_js(move || {
                    let _ = resolve.call1(&JsValue::UNDEFINED, &req_ok.result().unwrap());
                });
                req.clone().set_onsuccess(Some(onsuccess.unchecked_ref()));
                let onerror = Closure::once_into_js(move || {
                    let _ = reject.call1(&JsValue::UNDEFINED, &JsValue::from_str("open failed"));
                });
                req.set_onerror(Some(onerror.unchecked_ref()));
            })
        };
        let db: IdbDatabase = JsFuture::from(promise)
            .await
            .map_err(|e| format!("open db failed: {e:?}"))?
            .dyn_into()
            .map_err(|_| "result not a database".to_string())?;

        std::mem::forget(onupgradeneeded);
        Ok(Store { db })
    }

    fn store(&self, mode: IdbTransactionMode) -> Result<(IdbObjectStore, web_sys::IdbTransaction), String> {
        let tx = match mode {
            IdbTransactionMode::Readwrite => self
                .db
                .transaction_with_str_and_mode(STORE, IdbTransactionMode::Readwrite)
                .map_err(|e| format!("tx failed: {e:?}")),
            _ => self
                .db
                .transaction_with_str(STORE)
                .map_err(|e| format!("tx failed: {e:?}")),
        }?;
        let store = tx.object_store(STORE).map_err(|e| format!("no store: {e:?}"))?;
        Ok((store, tx))
    }

    pub async fn all(&self) -> Result<Vec<Code>, String> {
        let (store, _tx) = self.store(IdbTransactionMode::Readonly)?;
        let req = store.get_all().map_err(|e| format!("get_all: {e:?}"))?;
        let value = request_to_future(req)?.await.map_err(|e| format!("get_all: {e:?}"))?;
        let arr: Array = value.dyn_into().map_err(|_| "not an array".to_string())?;
        let mut codes = Vec::with_capacity(arr.length() as usize);
        for item in arr.iter() {
            if let Ok(code) = serde_wasm_bindgen::from_value::<Code>(item) {
                codes.push(code);
            }
        }
        Ok(codes)
    }

    pub async fn put(&self, code: &Code) -> Result<(), String> {
        let (store, _tx) = self.store(IdbTransactionMode::Readwrite)?;
        let value = serde_wasm_bindgen::to_value(code).map_err(|e| format!("serialize: {e}"))?;
        let key = JsValue::from_str(&code_key(code.symbology, &code.value));
        let req = store
            .put_with_key(&value, &key)
            .map_err(|e| format!("put: {e:?}"))?;
        request_to_future(req)?.await.map_err(|e| format!("put: {e:?}"))?;
        Ok(())
    }

    pub async fn delete(&self, symbology: Symbology, value: &str) -> Result<(), String> {
        let (store, _tx) = self.store(IdbTransactionMode::Readwrite)?;
        let key = JsValue::from_str(&code_key(symbology, value));
        let req = store.delete(&key).map_err(|e| format!("delete: {e:?}"))?;
        request_to_future(req)?.await.map_err(|e| format!("delete: {e:?}"))?;
        Ok(())
    }
}
