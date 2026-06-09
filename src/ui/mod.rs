use crate::filter::filter;
use crate::generate::generate;
use crate::get_pdf_ids;
use crate::profiles::settings::set_profile_location;
use crate::profiles::{delete_profile, set_profiles, Profile};
use crate::read_ids::get_csv_ids;
use native_dialog::DialogBuilder;
use slint::{ModelRc, SharedString, ToSharedString, VecModel};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::thread;



slint::include_modules!();
pub fn ui(pdf_directory: String, csv_file: String, profiles: Vec<Profile>) {
    println!("{}", pdf_directory);
    let ui = AppWindow::new().expect("Ouch, slint somehow didn't create the window");

    let csv_path = Rc::new(RefCell::new(PathBuf::from(csv_file)));
    let pdf_dir = Rc::new(RefCell::new(PathBuf::from(pdf_directory))); // had to use RefCell, to avoid ownership issues
    let all_pdf_names = Rc::new(RefCell::new(Vec::<String>::new()));
    let all_csv_names = Rc::new(RefCell::new(Vec::<String>::new()));
    let last_pattern = Rc::new(RefCell::new(String::new()));
    let last_column = Rc::new(RefCell::new(String::from("Buchungstext")));
    let name = Rc::new(RefCell::new(String::new()));
    let mut initial_names: Vec<SharedString> = profiles.iter().map(|p| SharedString::from(p.name.as_str())).collect();
    initial_names.push(SharedString::from("Neu"));
    Rc::new(RefCell::new(initial_names.clone()));

    let ui_handle = ui.as_weak();

    let update_profiles = {
        let ui_handle = ui_handle.clone();
        Rc::new(RefCell::new(move |profiles: Vec<Profile>| {
            let ui = ui_handle.unwrap();
            let n: Vec<SharedString> = profiles.iter()
                .map(|p| SharedString::from(p.name.as_str()))
                .collect();
            let mut n_with_neu = n.clone();
            n_with_neu.push(SharedString::from("Neu"));
            AppState::get(&ui).set_profiles(ModelRc::new(VecModel::from(n_with_neu)));
        }))
    };

    AppState::get(&ui).set_profile(initial_names[0].clone());

    AppState::get(&ui).set_profiles(ModelRc::new(VecModel::from(initial_names)));
    AppState::get(&ui).set_csv(csv_path.borrow().display().to_shared_string());
    AppState::get(&ui).set_pdfs(pdf_dir.borrow().display().to_shared_string());

    ui.on_request_csv_file({
        let ui_handle = ui.as_weak();
        let csv_path = Rc::clone(&csv_path);
        move || {
            let ui = ui_handle.unwrap();
            *csv_path.borrow_mut() = file_picker(String::from("csv"));

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
        let profiles = profiles.clone();
        move || {
            let csv_path = Rc::clone(&csv_path_clone).borrow().clone();
            let pdf_dir = Rc::clone(&pdf_dir_clone).borrow().clone();

            print!("generating");
            if !csv_path.exists() && !pdf_dir.exists() {
                return;
            }
            let ui = ui_handle.unwrap();
            let pdf_names = get_pdf_ids(pdf_dir_clone.borrow().clone());
            let csv_names = get_csv_ids(csv_path_clone.borrow().clone(), String::from("Buchungstext"));

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

            let profile = AppState::get(&ui).get_profile();
            if profile == "Neu" {
                AppState::get(&ui).set_current_view(1);
            } else {
                AppState::get(&ui).set_current_view(2);
                let p = profiles.iter().find(|val| val.name == profile.to_string()).unwrap();
                generate(pdf_dir, csv_path, p.pattern.clone(), p.column.clone());
            }
        }});

    ui.on_change_profiles_location(|| {
        let profile_location = file_picker(String::from("json"));
        set_profile_location(profile_location);
        println!("Profiles location changed!");
    });

    let ui_handle = ui.as_weak();
    ui.on_generate({
        let last_pattern_ref = Rc::clone(&last_pattern);
        let last_column_ref = Rc::clone(&last_column);
        println!("{:?}",last_column_ref);
        let name_ref = Rc::clone(&name);
        let ui_handle_ref = ui_handle.clone();
        let csv_path_clone = csv_path.clone();
        move || {
            let pdf_dir = pdf_dir.borrow().clone();
            let csv_path = csv_path_clone.borrow().clone();
            let last_pattern = last_pattern_ref.borrow().clone();
            let last_column = last_column_ref.borrow().clone();
            println!("{}", last_column);
            let ui_handle_clone = ui_handle_ref.clone();

            set_profiles(name_ref.borrow().clone(), last_pattern_ref.borrow().clone(), last_column_ref.borrow().clone()).expect("Couldn't save profile");

            thread::spawn(move || {
                print!("generating");
                if csv_path.exists() && pdf_dir.exists() { // TODO doesnt work for some reason?
                    generate(pdf_dir, csv_path, last_pattern, last_column);

                    slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_handle_clone.upgrade() {
                            AppState::get(&ui).set_current_view(2);
                        }
                    }).unwrap();
                }
            });
        }
    });

    ui.on_filter({
        let all_pdf_names = Rc::clone(&all_pdf_names);
        let ui_handle = ui.as_weak();
        let last_pattern = Rc::clone(&last_pattern);
        let all_csv_names = Rc::clone(&all_csv_names);

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

    ui.on_update_name({
        let name = name.clone();
        move |n: SharedString| {
            *name.borrow_mut() = String::from(n);
        }
    });


    ui.on_delete_profile({
        let ui_handle = ui_handle.clone();
        let update_profiles = update_profiles.clone();
        move || {
            let dialog = Rc::new(ConfirmDialog::new().unwrap());
            dialog.show().unwrap();

            dialog.on_confirm({
                let dialog = dialog.clone();
                let ui_handle = ui_handle.clone();
                let update_profiles = update_profiles.clone();
                move || {
                    let ui = ui_handle.unwrap();
                    let profile = AppState::get(&ui).get_profile();
                    let new_profiles = delete_profile(String::from(profile));
                    update_profiles.borrow_mut()(new_profiles);
                    dialog.hide().unwrap();
                }
            });

            dialog.on_abort({
                let dialog = dialog.clone();
                move || { dialog.hide().unwrap(); }
            });
        }
    });

    ui.on_update_ids( {
        let csv_path_clone = csv_path.clone();
        let ui_handle = ui.clone_strong();
        let column_ref = Rc::clone(&last_column);
        move |column| {
            *column_ref.borrow_mut() = column.to_string();
            let csv_names = get_csv_ids(csv_path_clone.borrow().clone(), String::from(column));
            AppState::get(&ui_handle).set_csv_names(
                ModelRc::new(VecModel::from(
                    csv_names.into_iter().map(|m| {
                        MatchedPattern { before: SharedString::from(m.to_string()), matched: SharedString::from(""), after: SharedString::from(""), partner: false }
                    }).collect::<Vec<MatchedPattern>>()
                ))
            );
        }
    });

    ui.run().expect("Failed to init window");
}

fn file_picker(file_type: String) -> PathBuf {
    let path = DialogBuilder::file()
        .add_filter(&file_type, [&file_type])
        .open_single_file()
        .show()
        .unwrap();

    path.unwrap_or_else(|| PathBuf::new())
}

fn directory_picker() -> PathBuf {
    let path = DialogBuilder::file()
        .open_single_dir()
        .show()
        .unwrap();

    path.unwrap_or_else(|| PathBuf::new())
}