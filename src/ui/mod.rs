use crate::filter::filter;
use crate::generate::generate;
use crate::get_ids::get_buchungsstapel_ids;
use crate::get_pdf_ids;
use crate::profiles::settings::set_profile_location;
use crate::profiles::{delete_profile, set_profiles, Profile};
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
            *csv_path.borrow_mut() = file_picker(Box::from(["csv", "scs"]));

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
            let csv_names = get_buchungsstapel_ids(csv_path_clone.borrow().clone(), String::from("Buchungstext"));

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

                let ui_clone = ui.as_weak();
                let pdf_dir_clone = pdf_dir.clone();
                let csv_path_clone = csv_path.clone();
                let profiles_clone = profiles.clone();
                let instant_import = AppState::get(&ui).get_instant_import();
                println!("instant import: {}", instant_import);

                thread::spawn(move || {
                    let p = profiles_clone.iter().find(|val| val.name == profile.to_string()).unwrap();

                    let total_buchungssaetze: usize;
                    let total_pdfs: usize;
                    let linked: i32;

                    (total_buchungssaetze, total_pdfs, linked) = generate(pdf_dir_clone, csv_path_clone, p.clone(), instant_import);

                    ui_clone.upgrade_in_event_loop(move |ui| {
                        AppState::get(&ui).set_total_buchungssaetze(total_buchungssaetze as i32);
                        AppState::get(&ui).set_total_pdfs(total_pdfs as i32);
                        AppState::get(&ui).set_linked(linked);
                        AppState::get(&ui).set_current_view(3);
                    }).unwrap();
                });
            }
        }
    });

    ui.on_change_profiles_location(|| {
        let profile_location = file_picker(Box::from(["json"]));
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
            let nummer = AppState::get(&ui_handle_clone.upgrade().unwrap()).get_mandantennummer();
            set_profiles(name_ref.borrow().clone(), last_pattern_ref.borrow().clone(), last_column_ref.borrow().clone(), nummer.to_string()).expect("Couldn't save profile");
            let name = name_ref.borrow().clone();
            AppState::get(&ui_handle_ref.upgrade().unwrap()).set_current_view(2);

            thread::spawn(move || {
                print!("generating");
                if csv_path.exists() && pdf_dir.exists() { // TODO doesnt work for some reason?
                    let total_buchungssaetze:usize;
                    let total_pdfs:usize;
                    let linked:i32;
                    let instant_import = AppState::get(&ui_handle_clone.clone().upgrade().unwrap()).get_instant_import();
                    let nummer = AppState::get(&ui_handle_clone.upgrade().unwrap()).get_mandantennummer();

                    (total_buchungssaetze, total_pdfs, linked) = generate(
                        pdf_dir,
                        csv_path,
                        Profile {
                            name,
                            pattern: last_pattern,
                            column: last_column,
                            nr: nummer.to_string(),
                        },
                        instant_import,
                    );

                    slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_handle_clone.upgrade() {
                            AppState::get(&ui).set_total_buchungssaetze(total_buchungssaetze as i32);
                            AppState::get(&ui).set_total_pdfs(total_pdfs as i32);
                            AppState::get(&ui).set_linked(linked);
                            AppState::get(&ui).set_current_view(3);
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
        let csv_name_ref = Rc::clone(&all_csv_names);
        move |column| {
            *column_ref.borrow_mut() = column.to_string();
            let csv_names = get_buchungsstapel_ids(csv_path_clone.borrow().clone(), String::from(column));
            *csv_name_ref.borrow_mut() = csv_names.clone();
            AppState::get(&ui_handle).set_csv_names(
                ModelRc::new(VecModel::from(
                    csv_names.into_iter().map(|m| {
                        MatchedPattern { before: SharedString::from(m.to_string()), matched: SharedString::from(""), after: SharedString::from(""), partner: false }
                    }).collect::<Vec<MatchedPattern>>()
                ))
            );
            ui_handle.invoke_filter(SharedString::from(<RefCell<String> as Clone>::clone(&Rc::clone(&last_pattern)).into_inner()));
        }
    });

    ui.run().expect("Failed to init window");
}

fn file_picker(file_types: Box<[&str]>) -> PathBuf {
    let mut builder = DialogBuilder::file();

    for ext in file_types.iter() {
        builder = builder.add_filter(ext.to_uppercase(), &[ext]);
    }

    let path = builder
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