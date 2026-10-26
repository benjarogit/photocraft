use super::*;
use egui_kittest::kittest::Queryable;

fn harness() -> egui_kittest::Harness<'static, PhotocraftApp> {
    let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1100.0, 1400.0)).with_max_steps(64).build_eframe(move |cc| {
        PhotocraftApp::setup_context(&cc.egui_ctx, Default::default());
        let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), crate::Services::default());
        app.deviantart = Settings { client_id: "99".into(), refresh_token: "synthetic".into(), scopes: "stash".into(), ..Default::default() };
        app
    });
    h.state_mut().run("file.new", json!({"width":2,"height":2})).unwrap();
    open_dialog(h.state_mut(), None);
    h.run_steps(5);
    h.get_by_label("More options").click();
    h.run_steps(4);
    h
}

fn capture(h: &mut egui_kittest::Harness<'_, PhotocraftApp>, name: &str) {
    if let Some(directory) = std::env::var_os("PHOTOCRAFT_DA_SURFACE_SCREENSHOTS") {
        let directory = std::path::PathBuf::from(directory);
        std::fs::create_dir_all(&directory).unwrap();
        h.render().unwrap().save(directory.join(format!("deviantart-{name}.png"))).unwrap();
    }
}

#[test]
fn publishing_is_independent_of_gallery_and_profile() {
    let mut h = harness();
    assert!(h.query_by_label("Publish").is_none());
    assert!(h.query_by_label("Get files").is_none());
    h.get_by_label("Publishing options").click();
    h.run_steps(4);
    h.get_by_label("Publish");
    h.get_by_label("Publish this publicly");
    assert!(h.query_by_label("Get files").is_none());
    assert!(h.query_by_label("Set as DeviantArt profile picture").is_none());
    h.get_by_label("Publish").scroll_to_me();
    h.run_steps(4);
    capture(&mut h, "publishing-only");
    // Merely opening this surface must not consent to publication.
    h.get_by_label("Publish").click();
    h.run_steps(3);
    assert_eq!(h.state().ui.status, "Confirm that this publish is public.");
    assert!(h.state().deviantart_job.is_none());
}

#[test]
fn management_is_independent_of_public_publishing() {
    let mut h = harness();
    h.get_by_label("Gallery and profile").click();
    h.run_steps(4);
    h.get_by_label("Get files");
    h.get_by_label("Set as DeviantArt profile picture");
    assert!(h.query_by_label("Publish").is_none());
    assert!(h.query_by_label("Publish this publicly").is_none());
    h.get_by_label("Set as DeviantArt profile picture").scroll_to_me();
    h.run_steps(4);
    capture(&mut h, "management-only");
}

#[test]
fn private_upload_and_public_publish_do_not_request_social_scopes() {
    assert_eq!(BASE_SCOPES, "stash");
    assert_eq!(requested_scopes("stash", extras::scopes_for_operation("publish")), "stash publish");
    assert_eq!(requested_scopes("stash", extras::scopes_for_operation("avatar")), "stash browse user.manage");
    assert_eq!(requested_scopes("stash", extras::scopes_for_operation("gallery")), "stash browse user");
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn operations_request_their_missing_scopes_before_calling_the_transport() {
    for operation in ["gallery", "download", "publish", "edit", "avatar"] {
        let (tx, rx) = std::sync::mpsc::channel();
        let services = crate::Services {
            deviantart_connect: Some(std::sync::Arc::new(move |_, scopes| {
                tx.send(scopes.to_string()).unwrap();
                Err("synthetic stopped login".into())
            })),
            deviantart_call: Some(std::sync::Arc::new(|_, _, _| panic!("operation ran before its scope was granted"))),
            ..Default::default()
        };
        let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), services);
        app.deviantart = Settings { client_id: "99".into(), refresh_token: "synthetic".into(), scopes: "stash".into(), ..Default::default() };
        extras::start_call(&mut app, &json!({"op":operation}).to_string()).unwrap();
        let requested = rx.recv_timeout(std::time::Duration::from_secs(2)).unwrap();
        assert_eq!(requested, requested_scopes("stash", extras::scopes_for_operation(operation)));
        assert!(matches!(app.deviantart_job.as_ref().unwrap().kind, JobKind::Connect));
    }
}

#[test]
fn independent_surface_labels_are_translated() {
    for lang in crate::i18n::Lang::all().filter(|l| l.complete_menus()) {
        for source in ["Publishing options", "Gallery and profile"] {
            assert_ne!(crate::i18n::tr(lang, source), source, "{}: {source}", lang.code());
        }
    }
}
