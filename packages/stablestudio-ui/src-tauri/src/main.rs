// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs::File;
use tauri_plugin_upload;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_upload::init())
        .invoke_handler(tauri::generate_handler![extract_zip, launch_comfy])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// tauri command to extract a zip from an arbitrary file path
#[tauri::command]
fn extract_zip(path: String, target_dir: String) -> Result<String, String> {
    println!("extracting zip from {} to {}", path, target_dir);
    let file = File::open(path).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();

    // unarchive in a thread
    let handle = std::thread::spawn(move || {
        archive.extract(target_dir).unwrap();
    });

    handle.join().unwrap();

    println!("extracted zip");
    Ok("completed".to_string())
}

// tauri command to launch python process
#[tauri::command]
fn launch_comfy(path: String) -> Result<String, String> {
    // check for cuda device
    let cuda_device = std::env::var("CUDA_VISIBLE_DEVICES").unwrap_or("".to_string());

    // set working directory
    std::env::set_current_dir(path.clone()).unwrap();

    // launch python process using embedded python executable (.exe on windows, .app on mac, etc.)
    let mut cmd = std::process::Command::new({
        if cfg!(unix) {
            "python_embeded/python"
        } else if cfg!(windows) {
            "python_embeded/python.exe"
        } else if cfg!(macos) {
            "python_embeded/python.app"
        } else {
            panic!("Unsupported platform")
        }
    });

    cmd.current_dir(path.clone());
    cmd.arg("-s");
    cmd.arg("ComfyUI/main.py");
    cmd.arg("--port");
    cmd.arg("5000");

    if cuda_device == "" {
        cmd.arg("--cpu");
    }

    if cfg!(windows) {
        cmd.arg("--windows-standalone-build");
    }

    println!("launching comfy: {:?}", cmd);

    let mut child = cmd.spawn().expect("failed to execute process");

    // run python process in background
    std::thread::spawn(move || {
        child.wait().expect("failed to wait on child");
    });

    Ok("completed".to_string())
}
