// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::sync::{
  Arc, Mutex,
  atomic::{AtomicI32, Ordering},
};

use cef::{Browser, ImplBrowser, ImplBrowserHost};

use crate::webview::{DevToolsProtocol, DevToolsProtocolHandler};

fn next_message_id() -> i32 {
  static NEXT_ID: AtomicI32 = AtomicI32::new(1000);
  NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

fn send(browser: &Browser, method: &str, params: serde_json::Value) {
  if let Some(host) = browser.host() {
    let message = serde_json::json!({
        "id": next_message_id(),
        "method": method,
        "params": params,
    })
    .to_string();
    host.send_dev_tools_message(Some(message.as_bytes()));
  }
}

/// Bypasses CSP for `browser`
pub(crate) fn arm_csp_bypass(
  browser: &Browser,
  handlers: &Arc<Mutex<Vec<Arc<DevToolsProtocolHandler>>>>,
) {
  let handler_browser = browser.clone();
  handlers.lock().unwrap().push(Arc::new(move |protocol: DevToolsProtocol| {
    let DevToolsProtocol::Event { method, .. } = protocol else {
      return;
    };
    if method == "Page.frameNavigated" {
      send(&handler_browser, "Page.setBypassCSP", serde_json::json!({ "enabled": true }));
    }
  }));

  send(browser, "Page.enable", serde_json::json!({}));
  send(browser, "Page.setBypassCSP", serde_json::json!({ "enabled": true }));
}
