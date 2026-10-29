pub use km003c_lib::runtime::*;

/// Whether [`sleep`] will be stretched by the browser.
///
/// Hidden tabs and unfocused windows clamp `setTimeout` to about one second.
/// A poll loop that sleeps between reads then lets the meter drop samples.
/// USB transfer completion is not a timer, so the caller can keep reading
/// without sleeping.
#[cfg(not(target_arch = "wasm32"))]
pub fn browser_timers_throttled() -> bool {
    false
}

#[cfg(target_arch = "wasm32")]
pub fn browser_timers_throttled() -> bool {
    web_sys::window()
        .and_then(|window| window.document())
        .is_some_and(|document| document.hidden() || !document.has_focus().unwrap_or(true))
}
