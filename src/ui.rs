#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use native_dialog::DialogBuilder;
use slint::ToSharedString;
use crate::generate;

slint::include_modules!();
pub fn ui() {
    let ui = AppWindow::new().expect("Ouch, slint somehow didn't create the window");
    let csv_path = Rc::new(RefCell::new(PathBuf::new()));
    let pdf_dir = Rc::new(RefCell::new(PathBuf::new())); // had to use RefCell, to avoid ownership issues

    ui.on_request_csv_file({
        let ui_handle = ui.as_weak();
        let csv_path = Rc::clone(&csv_path);
        move || {
            let ui = ui_handle.unwrap();
            *csv_path.borrow_mut() = file_picker();

            ui.set_csv(csv_path.borrow().display().to_shared_string())
        }
    });

    ui.on_request_pdf_dir({
        let pdf_dir = Rc::clone(&pdf_dir);
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            *pdf_dir.borrow_mut() = directory_picker();
            ui.set_pdfs(pdf_dir.borrow().display().to_shared_string())
        }
    });


    ui.on_generate({
        let csv_path = Rc::clone(&csv_path);
        let pdf_dir = Rc::clone(&pdf_dir);
        move || {
            generate(pdf_dir.borrow().clone(), csv_path.borrow().clone());
        }
    });

    ui.run().expect("Failed to init window");
}

fn file_picker() -> PathBuf {
    let path = DialogBuilder::file()
        .add_filter("csv", ["csv"])
        .open_single_file()
        .show()
        .unwrap();

    match path {
        Some(path) => path,
        None => return PathBuf::new(),
    }
}

fn directory_picker() -> PathBuf {
    let path = DialogBuilder::file()
        .open_single_dir()
        .show()
        .unwrap();

    match path {
        Some(path) => path,
        None => return PathBuf::new(),
    }
}