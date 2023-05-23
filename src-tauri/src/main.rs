// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::ffi::{OsStr};
use std::io;
use std::path::{Path};
use std::sync::{Mutex, Arc, RwLock};
use std::time::Duration;

use tauri::{SystemTray, SystemTraySubmenu};
use tauri::{CustomMenuItem, SystemTrayMenu, SystemTrayMenuItem, SystemTrayEvent, Icon};
use tauri::{Manager};

use sysinfo::{CpuExt, System, SystemExt};

#[derive(Debug)]
struct Pet {
    name: String,
    dark_icons: Vec<Icon>,
    light_icons: Vec<Icon>,
}

fn create_pets() -> io::Result<Vec<Pet>> {
    let pet_names = ["cat", "horse", "parrot"];
    let mut pets = Vec::new();

    for name in pet_names {
        let path_name = format!("icons/{name}");
        let path = Path::new(&path_name);
        if !path.exists() {
            continue;
        }

        if path.is_dir() {
            let mut dark_icons = Vec::new();
            let mut light_icons = Vec::new();

            for d in std::fs::read_dir(path)? {
                let path = d?.path();

                match path.extension().and_then(OsStr::to_str) {
                    Some("ico" | "png") => {
                        if let Some(stem) = path.file_stem().and_then(OsStr::to_str) {
                            if stem.contains("light") {
                                light_icons.push(tauri::Icon::File(path));
                            } else if stem.contains("dark") {
                                dark_icons.push(tauri::Icon::File(path));
                            }
                        }
                    },
                    _ => (),
                }
            }
            if light_icons.len() == 0 {
                println!("no icons found");
                continue;
            }
            if light_icons.len() != dark_icons.len() {
                println!("mismatched icon number between dark and light theme");
                continue;
            }
            let mut caption = name.to_string();
            caption.make_ascii_uppercase();

            pets.push(Pet {
                name: String::from(name),
                dark_icons,
                light_icons,
            });
        }
    }

    Ok(pets)
}

fn main() {
    let pets = Arc::new(RwLock::new(create_pets().unwrap()));
    let pet_selected = Arc::new(RwLock::new(0));
    let theme_selected = Mutex::new(0);

    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let setting = CustomMenuItem::new("setting".to_string(), "Setting");
    let mut sub_pet_menu = SystemTrayMenu::new();

    for pet in &*pets.read().unwrap() {
        let mut caption = pet.name.to_string();
        caption.make_ascii_uppercase();

        sub_pet_menu = sub_pet_menu.add_item(CustomMenuItem::new(pet.name.to_string(), caption));
    }

    let pet_menu = SystemTraySubmenu::new("Pet", sub_pet_menu);
    let tray_menu = SystemTrayMenu::new()
                                        .add_submenu(pet_menu)
                                        .add_item(setting)
                                        .add_native_item(SystemTrayMenuItem::Separator)
                                        .add_item(quit);
    let tray = SystemTray::new().with_menu(tray_menu);
    let hidden = Mutex::new(true);

    let _pets = pets.clone();
    let _pet_selected = pet_selected.clone();

    tauri::Builder::default()
        .setup(|app| {
            let _app = app.handle();

            std::thread::spawn(move || {
                let mut sys = System::new_all();
                let mut acc_time = 2800_usize;  /* 200ms required to get an accurate result */
                let mut selected = *_pet_selected.read().unwrap();

                sys.refresh_cpu();

                loop {
                    for icon in &(*_pets.read().unwrap())[selected].dark_icons {
                        let mut interval = 200_f64;
                        if acc_time >= 3000 {
                            let cpu_usage;

                            sys.refresh_cpu();
                            cpu_usage = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() as f64 / sys.cpus().len() as f64;
                            interval = 200.0_f64 / 1.0_f64.max(20.0_f64.min(cpu_usage / 5.0_f64));

                            acc_time = 0;
                        }
                        _app.tray_handle().set_icon(icon.clone()).unwrap();
                        std::thread::sleep(Duration::from_millis(interval as u64));
                        selected = *_pet_selected.read().unwrap();
                        acc_time += interval as usize;
                    }
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
                _ => {
                    for pet in pets.read().unwrap().iter().enumerate() {
                        if pet.1.name == id {
                            *pet_selected.write().unwrap() = pet.0;
                            break;
                        }
                    }
                }
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
