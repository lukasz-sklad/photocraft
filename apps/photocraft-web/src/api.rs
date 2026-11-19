//! `window.photocraft`: the control channel for the page, as promises and live event syncing.

use std::cell::RefCell;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};

use photocraft_ui_egui::theme::{ThemeKind, Tokens};
use photocraft_ui_egui::ControlRequest;
use serde_json::{json, Value};
use wasm_bindgen::prelude::*;

use crate::web::Inbox;

type Waiting = (Receiver<Value>, js_sys::Function, js_sys::Function);

thread_local! {
    static TX: RefCell<Option<Sender<ControlRequest>>> = const { RefCell::new(None) };
    static WAITING: RefCell<Vec<Waiting>> = const { RefCell::new(Vec::new()) };
    static CTX: RefCell<Option<egui::Context>> = const { RefCell::new(None) };
    static INBOX: RefCell<Option<Inbox>> = const { RefCell::new(None) };
}

pub fn connect(tx: Option<Sender<ControlRequest>>, inbox: Inbox, ctx: egui::Context) {
    if let Some(tx) = tx {
        TX.with(|t| *t.borrow_mut() = Some(tx));
    }
    INBOX.with(|i| *i.borrow_mut() = Some(inbox));
    CTX.with(|c| *c.borrow_mut() = Some(ctx));
}

pub fn repaint() {
    CTX.with(|c| {
        if let Some(ctx) = c.borrow().as_ref() {
            ctx.request_repaint();
        }
    });
}

fn to_js(v: &Value) -> JsValue {
    js_sys::JSON::parse(&v.to_string()).unwrap_or(JsValue::NULL)
}

fn from_js(v: &JsValue) -> Value {
    if v.is_undefined() || v.is_null() {
        return json!({});
    }
    js_sys::JSON::stringify(v)
        .ok()
        .and_then(|s| s.as_string())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(json!({}))
}

pub fn poll_replies() {
    WAITING.with(|w| {
        w.borrow_mut().retain(|(rx, resolve, reject)| match rx.try_recv() {
            Ok(v) => {
                if v.get("ok").and_then(Value::as_bool) == Some(true) {
                    let _ = resolve.call1(&JsValue::NULL, &to_js(v.get("result").unwrap_or(&Value::Null)));
                } else {
                    let msg = v.get("error").and_then(Value::as_str).unwrap_or("error").to_string();
                    let _ = reject.call1(&JsValue::NULL, &js_sys::Error::new(&msg));
                }
                false
            }
            Err(TryRecvError::Empty) => true,
            Err(TryRecvError::Disconnected) => {
                let _ = reject.call1(&JsValue::NULL, &js_sys::Error::new("request dropped"));
                false
            }
        });
    });
}

pub fn request(method: &str, params: Value) -> js_sys::Promise {
    let method = method.to_string();
    let p = js_sys::Promise::new(&mut |resolve, reject| {
        let (req, rx) = ControlRequest::new(method.clone(), params.clone());
        let sent = TX.with(|t| t.borrow().as_ref().is_some_and(|tx| tx.send(req).is_ok()));
        if sent {
            WAITING.with(|w| w.borrow_mut().push((rx, resolve, reject)));
        } else {
            let _ = reject.call1(&JsValue::NULL, &js_sys::Error::new("PhotoCraft is not running yet"));
        }
    });
    repaint();
    p
}

fn set_fn(obj: &js_sys::Object, name: &str, f: JsValue) -> Result<(), JsValue> {
    js_sys::Reflect::set(obj, &name.into(), &f).map(|_| ())
}

pub fn theme_for(theme: &str) -> Option<ThemeKind> {
    ThemeKind::from_name(theme)
}

pub fn update_body_background(kind: ThemeKind) {
    if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
        if let Some(body) = doc.body() {
            let (bg, text_color) = match kind {
                ThemeKind::Pro => ("#262626", "#b8b8b8"),
                ThemeKind::ProMedium => ("#333333", "#c8c8c8"),
                ThemeKind::ProLightGray => ("#b8b8b8", "#1f1710"),
                ThemeKind::ProLight => ("#f0f0f0", "#333333"),
                ThemeKind::Studio => ("#1c1c1f", "#cccccc"),
                ThemeKind::StudioLight => ("#f5f5f7", "#222222"),
                ThemeKind::Classic => ("#d4d0c8", "#111111"),
                _ => {
                    if Tokens::for_kind(kind).dark() {
                        ("#262626", "#b8b8b8")
                    } else {
                        ("#f0f0f0", "#333333")
                    }
                }
            };
            let _ = body.style().set_property("background", bg);
            if let Some(loading) = doc.get_element_by_id("photocraft_loading") {
                if let Ok(loading_el) = loading.dyn_into::<web_sys::HtmlElement>() {
                    let _ = loading_el.style().set_property("color", text_color);
                }
            }
        }
    }
}

pub fn send_theme(kind: ThemeKind) {
    let (req, _rx) = ControlRequest::new("ui.set", json!({ "theme": kind.id() }));
    let sent = TX.with(|t| t.borrow().as_ref().is_some_and(|tx| tx.send(req).is_ok()));
    if sent {
        update_body_background(kind);
        repaint();
    }
}

pub fn send_lang(lang: &str) {
    let (req, _rx) = ControlRequest::new(
        "engine.execute",
        json!({
            "command": "prefs.set",
            "params": {
                "path": "interface.language",
                "value": lang
            }
        }),
    );
    let sent = TX.with(|t| t.borrow().as_ref().is_some_and(|tx| tx.send(req).is_ok()));
    if sent {
        repaint();
    }
}

pub fn install() -> Result<(), JsValue> {
    let obj = js_sys::Object::new();
    set_fn(
        &obj,
        "request",
        Closure::<dyn Fn(String, JsValue) -> js_sys::Promise>::new(|method: String, params: JsValue| request(&method, from_js(&params)))
            .into_js_value(),
    )?;
    set_fn(
        &obj,
        "execute",
        Closure::<dyn Fn(String, JsValue) -> js_sys::Promise>::new(|command: String, params: JsValue| {
            request("engine.execute", json!({"command": command, "params": from_js(&params)}))
        })
        .into_js_value(),
    )?;
    set_fn(
        &obj,
        "setTheme",
        Closure::<dyn Fn(String) -> js_sys::Promise>::new(|theme_name: String| {
            if let Some(kind) = theme_for(&theme_name) {
                update_body_background(kind);
            }
            request("ui.set", json!({ "theme": theme_name }))
        })
        .into_js_value(),
    )?;
    set_fn(&obj, "commands", Closure::<dyn Fn() -> js_sys::Promise>::new(|| request("engine.commands", json!({}))).into_js_value())?;
    set_fn(&obj, "inspect", Closure::<dyn Fn() -> js_sys::Promise>::new(|| request("ui.inspect", json!({}))).into_js_value())?;
    set_fn(
        &obj,
        "openFile",
        Closure::<dyn Fn(String, js_sys::Uint8Array) -> JsValue>::new(|name: String, bytes: js_sys::Uint8Array| {
            let queued = INBOX.with(|i| {
                i.borrow().as_ref().is_some_and(|inbox| {
                    inbox.lock().unwrap_or_else(|e| e.into_inner()).push((name, bytes.to_vec()));
                    true
                })
            });
            repaint();
            JsValue::from_bool(queued)
        })
        .into_js_value(),
    )?;

    let w = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
    js_sys::Reflect::set(&w, &"photocraft".into(), &obj)?;

    let on_msg = Closure::<dyn FnMut(web_sys::Event)>::new(move |e: web_sys::Event| {
        let Ok(raw_data) = js_sys::Reflect::get(&e, &"data".into()) else { return };
        let data = from_js(&raw_data);
        let msg_type = data.get("type").and_then(Value::as_str).unwrap_or("");

        let theme_str = if let Some(s) = raw_data.as_string() {
            Some(s)
        } else {
            data.get("theme")
                .or_else(|| data.get("value"))
                .and_then(Value::as_str)
                .map(|s| s.to_string())
        };

        if matches!(msg_type, "set_theme" | "craft-theme") || (msg_type.is_empty() && theme_str.is_some()) {
            if let Some(theme) = theme_str
                && let Some(kind) = theme_for(&theme)
            {
                send_theme(kind);
            }
        } else if matches!(msg_type, "set_language" | "craft-lang") {
            if let Some(lang) = data.get("lang").or_else(|| data.get("language")).and_then(Value::as_str) {
                send_lang(lang);
            }
        }
    });
    let _ = w.add_event_listener_with_callback("message", on_msg.as_ref().unchecked_ref());
    on_msg.forget();

    Ok(())
}
