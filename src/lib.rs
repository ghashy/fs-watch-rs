#![allow(dead_code)]

use std::path::Path;
use std::sync::mpsc::{self, Receiver};

use godot::obj::WithBaseField;
use godot::prelude::Dictionary;
use godot::prelude::*;

use notify::{Event, Watcher};
use notify::{RecursiveMode, Result};

#[cfg(target_os = "macos")]
use notify::FsEventWatcher;

#[cfg(any(target_os = "linux", target_os = "android"))]
use notify::INotifyWatcher;

#[cfg(target_os = "windows")]
use notify::ReadDirectoryChangesWatcher;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}

#[derive(GodotClass)]
#[class(no_init, base = Object)]
struct FsWatcher {
    #[cfg(target_os = "macos")]
    watcher: Option<FsEventWatcher>,
    #[cfg(any(target_os = "linux", target_os = "android"))]
    watcher: Option<INotifyWatcher>,
    #[cfg(target_os = "windows")]
    watcher: Option<ReadDirectoryChangesWatcher>,
    rx: Receiver<Event>,
    base: Base<Object>,
}

#[godot_api]
impl FsWatcher {
    #[func]
    fn from_paths(paths: Array<GString>) -> Gd<Self> {
        let (tx, rx) = mpsc::channel();

        let mut watcher =
            notify::recommended_watcher(move |res: Result<Event>| {
                if let Ok(event) = res {
                    tx.send(event)
                        .expect("Failed to send filesystem notification");
                }
            })
            .unwrap();

        for path in paths.iter_shared() {
            watcher
                .watch(Path::new(&path.to_string()), RecursiveMode::Recursive)
                .unwrap();
        }

        Gd::from_init_fn(|base| Self {
            base,
            watcher: Some(watcher),
            rx,
        })
    }

    #[func]
    fn start(&mut self) {
        loop {
            match self.rx.recv() {
                Ok(event) => {
                    let dict = event_to_dict(event);
                    self.base_mut().emit_signal("changed".into(), &[dict]);
                }
                Err(err) => {
                    godot_print!("Closing channel with: {err}");
                    break;
                }
            }
        }
    }

    #[func]
    fn stop(&mut self) {
        self.watcher = None;
    }

    #[signal]
    fn changed() {}
}

use serde_json::Value;

fn value_to_variant(value: Value) -> Variant {
    match value {
        Value::Null => Variant::nil(),
        Value::Bool(v) => v.to_variant(),
        Value::Number(v) => {
            if v.is_u64() {
                v.as_u64().unwrap().to_variant()
            } else if v.is_i64() {
                v.as_i64().unwrap().to_variant()
            } else {
                v.as_f64().unwrap().to_variant()
            }
        }
        Value::String(v) => v.to_variant(),
        Value::Array(arr) => {
            let mut nested = VariantArray::new();
            for item in arr {
                nested.push(value_to_variant(item));
            }
            nested.to_variant()
        }
        Value::Object(obj) => {
            let mut nested = Dictionary::new();
            for (key, value) in obj {
                nested.insert(key.to_variant(), value_to_variant(value));
            }
            nested.to_variant()
        }
    }
}

fn event_to_dict(event: Event) -> Variant {
    let json = serde_json::to_value(&event).unwrap();
    value_to_variant(json)
}

// fn event_to_dict(event: Event) -> Dictionary {
//     use serde_json::Map;
//     let json = serde_json::to_value(&event).unwrap();

//     let mut dict = Dictionary::new();

//     dict.insert(Variant::from("key"), Variant::from(Dictionary::new()));

//     todo!()
// }

// fn push_object(
//     dict: &mut Dictionary,
//     map: serde_json::Map<String, serde_json::Value>,
// ) {
//     let mut nested = Dictionary::new();
//     for (key, value) in map {
//         let key = key.to_variant();
//         match value {
//             serde_json::Value::Null => {
//                 nested.insert(key, Variant::nil());
//             }
//             serde_json::Value::Bool(b) => {
//                 nested.insert(key, b.to_variant());
//             }
//             serde_json::Value::Number(_) => {
//                 let v: f64 = serde_json::from_value(value).unwrap();
//                 nested.insert(key, v.to_variant());
//             }
//             serde_json::Value::String(s) => {
//                 nested.insert(key, s.to_variant());
//             }
//             serde_json::Value::Array(arr) => {
//                 push_arr(&mut nested, key, arr);
//             }
//             serde_json::Value::Object(obj) => {
//                 push_object(&mut nested, obj);
//             }
//         };
//     }
// }

// fn push_arr(
//     dict: &mut Dictionary,
//     key: Variant,
//     arr: Vec<serde_json::Value>,
// ) -> Option<Variant> {
//     let mut nested = Array::new();
//     for value in arr {
//         match value {
//             serde_json::Value::Null => nested.push(Variant::nil()),
//             serde_json::Value::Bool(b) => nested.push(b.to_variant()),
//             serde_json::Value::Number(_) => {
//                 let v: f64 = serde_json::from_value(value).unwrap();
//                 nested.push(v.to_variant())
//             }
//             serde_json::Value::String(s) => nested.push(s.to_variant()),
//             serde_json::Value::Array(arr) => {
//                 push_arr(dict, key.clone(), arr);
//             }
//             serde_json::Value::Object(obj) => {
//                 let mut dict = Dictionary::new();
//                 push_object(&mut dict, obj);
//                 nested.push(dict.to_variant());
//             }
//         };
//     }
//     dict.insert(key, nested)
// }
