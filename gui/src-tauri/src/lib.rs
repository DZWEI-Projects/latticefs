#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_dialog::init())
    .invoke_handler(tauri::generate_handler![
      crate::commands::get_repo_info,
      crate::commands::init_repo,
      crate::commands::check_initialized,
      crate::commands::import_paths,
      crate::commands::create_sample_files,
      crate::commands::get_onboarding_graph,
      crate::commands::list_views,
      crate::commands::get_view_objects,
      crate::commands::evaluate_query,
      crate::commands::add_object_tag,
      crate::commands::remove_object_tag,
      crate::commands::set_object_trust_level,
      crate::commands::open_object,
      crate::commands::create_view,
      crate::commands::update_view,
      crate::commands::delete_view,
      crate::commands::list_object_versions,
      crate::commands::get_object_version_text,
      crate::commands::diff_object_versions,
      crate::commands::revise_object_from_text,
      crate::commands::revise_object_from_file,
      crate::commands::set_version_state,
      crate::commands::checkout_object_version,
      crate::commands::export_object_version,
    ])
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

mod commands;
