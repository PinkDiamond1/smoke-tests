use std::boxed::Box;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use web_view::Handle;

struct EventHandler {
  on_event: Box<dyn FnMut(String)>,
}

thread_local!(static LISTENERS: Arc<Mutex<HashMap<String, EventHandler>>> = Arc::new(Mutex::new(HashMap::new())));

lazy_static! {
  static ref EMIT_FUNCTION_NAME: String = uuid::Uuid::new_v4().to_string();
  static ref EVENT_LISTENERS_OBJECT_NAME: String = uuid::Uuid::new_v4().to_string();
  static ref EVENT_QUEUE_OBJECT_NAME: String = uuid::Uuid::new_v4().to_string();
}

pub fn emit_function_name() -> String {
  EMIT_FUNCTION_NAME.to_string()
}

pub fn event_listeners_object_name() -> String {
  EVENT_LISTENERS_OBJECT_NAME.to_string()
}

pub fn event_queue_object_name() -> String {
  EVENT_QUEUE_OBJECT_NAME.to_string()
}

pub fn listen<F: FnMut(String) + 'static>(id: &'static str, handler: F) {
  LISTENERS.with(|listeners| {
    let mut l = listeners
      .lock()
      .expect("Failed to lock listeners: listen()");
    l.insert(
      id.to_string(),
      EventHandler {
        on_event: Box::new(handler),
      },
    );
  });
}

pub fn emit<T: 'static>(webview_handle: &Handle<T>, event: &'static str, mut payload: String) {
  let salt = crate::salt::generate();
  if payload == "" {
    payload = "void 0".to_string();
  }

  webview_handle
    .dispatch(move |_webview| {
      _webview.eval(&format!(
        "window['{}']({{type: '{}', payload: {}}}, '{}')",
        emit_function_name(),
        event,
        payload,
        salt
      ))
    })
    .expect("Failed to dispatch JS from emit");
}

pub fn on_event(event: String, data: String) {
  LISTENERS.with(|listeners| {
    let mut l = listeners
      .lock()
      .expect("Failed to lock listeners: on_event()");

    let key = event.clone();

    if l.contains_key(&key) {
      let handler = l.get_mut(&key).expect("Failed to get mutable handler");
      (handler.on_event)(data);
    }
  });
}
