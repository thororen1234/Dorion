use crate::injection::injection_runner;
use std::time::Duration;

// Global "is injected" var
static mut IS_READY: bool = false;

pub fn inject_routine(win: tauri::Window) {
  let win_cln = win.clone();

  win.once("initial_inject", move |_| unsafe {
    IS_READY = true;
    println!("JS context ready!");

    #[cfg(target_os = "linux")]
    {
      // Check if webrtc is enabled
      win_cln.with_webview(|webview| {
        use webkit2gtk::{WebViewExt, SettingsExt};
  
        let wv = webview.inner();
        let wv = wv.as_ref();
        let settings = WebViewExt::settings(wv).unwrap_or_default();

        // Voice and video
        println!("Is WebRTC enabled? {}", settings.enables_webrtc());

        settings.set_enable_webrtc(true);

        wv.set_settings(&settings);
      }).unwrap_or_else(|_| println!("Failed to check WebRTC status"));
    }

    injection_runner::do_injection(win_cln);
  });

  std::thread::spawn(move || {
    loop {
      unsafe {
        if IS_READY {
          break;
        }
      }

      println!("JS context not ready...");

      // Send javascript that sends the "initial_inject" event
      //
      // If it succeeds, that means the web context is ready
      win
        .eval("window.__TAURI__.event.emit('initial_inject')")
        .unwrap();

      std::thread::sleep(Duration::from_millis(10));
    }
  });
}
