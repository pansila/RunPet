// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::{PathBuf};
use std::str::FromStr;
use std::sync::{Mutex};
use std::time::Duration;

use tauri::SystemTray;
use tauri::{CustomMenuItem, SystemTrayMenu, SystemTrayMenuItem, SystemTrayEvent};
use tauri::{Manager};

use sysinfo::{CpuExt, System, SystemExt};

fn main() {
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let setting = CustomMenuItem::new("setting".to_string(), "Setting");
    let tray_menu = SystemTrayMenu::new()
                                        .add_item(quit)
                                        .add_native_item(SystemTrayMenuItem::Separator)
                                        .add_item(setting);
    let tray = SystemTray::new().with_menu(tray_menu);
    let hidden = Mutex::new(true);
    let icons = Mutex::new(Vec::new());

    for i in 0..5 {
        icons.lock().unwrap().push(tauri::Icon::File(PathBuf::from_str(&format!("icons/cat/dark_cat_{i}.ico")).unwrap()));
    }

    tauri::Builder::default()
        .setup(|app| {
            let _app = app.handle();
            std::thread::spawn(move || {
                let mut sys = System::new_all();
                let mut acc_time = 2800_usize;  /* 200ms required to get an accurate result */

                sys.refresh_cpu();

                for icon in icons.lock().unwrap().iter().cycle() {
                    let mut interval = 200_f64;
                    if acc_time >= 3000 {
                        let cpu_usage;

                        sys.refresh_cpu();
                        cpu_usage = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() as f64 / sys.cpus().len() as f64;
                        interval = 200.0_f64 / 1.0_f64.max(20.0_f64.min(cpu_usage / 5.0_f64));

                        acc_time = 0;
                    }
                    // println!("3333 {}", interval);
                    _app.tray_handle().set_icon(icon.clone()).unwrap();
                    std::thread::sleep(Duration::from_millis(interval as u64));
                    acc_time += interval as usize;
                }
            });
 
            Ok(())
        })
        .system_tray(tray)
        .on_system_tray_event(move |app, event| match event {
            SystemTrayEvent::MenuItemClick { id, .. } => {
              match id.as_str() {
                "quit" => {
                  std::process::exit(0);
                }
                "setting" => {
                  let window = app.get_window("main").unwrap();
                  let _hidden;
                  {
                    if *hidden.lock().unwrap() {
                      window.show().unwrap();
                      _hidden = false;
                    } else {
                      window.hide().unwrap();
                      _hidden = true;
                    }
                  }
                  *hidden.lock().unwrap() = _hidden;
                }
                _ => {}
              }
            }
            _ => {}
          })
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_app, event| match event {
            tauri::RunEvent::ExitRequested { api, .. } => {
              api.prevent_exit();
            }
            _ => {}
          });
}
