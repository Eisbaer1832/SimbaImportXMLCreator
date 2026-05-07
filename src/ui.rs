#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::filter::filter;
use crate::read_ids::get_csv_ids;
use crate::{generate, get_pdf_ids};
use native_dialog::DialogBuilder;
use slint::{ModelRc, SharedString, ToSharedString, VecModel};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

slint::include_modules!();
pub fn ui() {
    let ui = AppWindow::new().expect("Ouch, slint somehow didn't create the window");
    let csv_path = Rc::new(RefCell::new(PathBuf::new()));
    let pdf_dir = Rc::new(RefCell::new(PathBuf::new())); // had to use RefCell, to avoid ownership issues
    let all_pdf_names = Rc::new(RefCell::new(Vec::<String>::new()));
    let all_csv_names = Rc::new(RefCell::new(Vec::<String>::new()));
    let last_pattern = Rc::new(RefCell::new(String::new()));

    ui.on_request_csv_file({
        let ui_handle = ui.as_weak();
        let csv_path = Rc::clone(&csv_path);
        move || {
            let ui = ui_handle.unwrap();
            *csv_path.borrow_mut() = file_picker();

            AppState::get(&ui).set_csv(csv_path.borrow().display().to_shared_string())
        }
    });

    ui.on_request_pdf_dir({
        let pdf_dir = Rc::clone(&pdf_dir);
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            *pdf_dir.borrow_mut() = directory_picker();
            AppState::get(&ui).set_pdfs(pdf_dir.borrow().display().to_shared_string())
        }
    });


    let pdf_dir_clone = pdf_dir.clone();
    let csv_path_clone = csv_path.clone();

    let ui_handle = ui.as_weak();

    ui.on_get_names({
        let all_pdf_names = Rc::clone(&all_pdf_names);
        let all_csv_names = Rc::clone(&all_csv_names);

        move || {
            let ui = ui_handle.unwrap();
            let pdf_names = get_pdf_ids(pdf_dir_clone.borrow().clone());
            let csv_names = get_csv_ids(csv_path_clone.borrow().clone());

            *all_pdf_names.borrow_mut() = pdf_names.clone();
            *all_csv_names.borrow_mut() = csv_names.clone();


            AppState::get(&ui).set_pdf_names(
                ModelRc::new(VecModel::from(
                    pdf_names.into_iter().map(|m| {
                        MatchedPattern { before: SharedString::from(m.to_string()), matched: SharedString::from(""), after: SharedString::from(""), partner: false}
                    }).collect::<Vec<MatchedPattern>>()
                ))
            );

            AppState::get(&ui).set_csv_names(
                ModelRc::new(VecModel::from(
                    csv_names.into_iter().map(|m| {
                        MatchedPattern { before: SharedString::from(m.to_string()), matched: SharedString::from(""), after: SharedString::from(""), partner: false }
                    }).collect::<Vec<MatchedPattern>>()
                ))
            );
            AppState::get(&ui).set_current_view(1);
        }});


    ui.on_generate({
        let csv_path = Rc::clone(&csv_path);
        let pdf_dir = Rc::clone(&pdf_dir);
        let last_pattern = Rc::clone(&last_pattern);
        move || {
            generate(pdf_dir.borrow().clone(), csv_path.borrow().clone(),last_pattern.borrow().clone());
        }
    });

    ui.on_filter({
        let all_pdf_names = Rc::clone(&all_pdf_names);
        let ui_handle = ui.as_weak();
        let last_pattern = Rc::clone(&last_pattern);

        move |pattern: SharedString| {
            let ui = ui_handle.unwrap();
            *last_pattern.borrow_mut() = String::from(&*pattern);

            let mut pdfs = filter(all_pdf_names.borrow().clone(), &*pattern);
            let mut csvs = filter(all_csv_names.borrow().clone(), &*pattern);

            for pdf in pdfs.iter_mut() {
                if let Some(m) = csvs.iter_mut().find(|csv| csv.matched == pdf.matched) {
                    pdf.partner = true;
                    m.partner = true;
                }
            }

            AppState::get(&ui).set_pdf_names(
                ModelRc::new(VecModel::from(pdfs))
            );
            AppState::get(&ui).set_csv_names(
                ModelRc::new(VecModel::from(csvs))
            );
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

    path.unwrap_or_else(|| PathBuf::new())
}