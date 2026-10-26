//! Submission dialog and optional gallery/profile controls.
use super::extras::{avatar_file, gallery_section, start_call};
use super::*;

pub(super) fn body(app: &mut PhotocraftApp, ui: &mut egui::Ui, f: &mut serde_json::Map<String, Value>) {
    let t = crate::theme::Tokens::get(ui.ctx());
    ui.set_min_width(440.0);
    ui.label(egui::RichText::new(tl!("Submitting needs an internet connection.")).color(t.text_dim));
    ui.add_space(4.0);
    ui.label(
        egui::RichText::new(tl!(
            "Create a Public app on DeviantArt, without a client secret. Copy the address below into the redirect field, paste the client id here, then connect."
        ))
        .color(t.text_dim),
    );
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.monospace(REDIRECT_URI);
        if crate::widgets::secondary_button(ui, tl!("Copy"), 72.0).clicked() {
            ui.ctx().copy_text(REDIRECT_URI.to_string());
            app.ui.status = tl!("Copied the redirect address.").to_string();
            app.ui.status_error = false;
        }
    });
    ui.add_space(4.0);
    if crate::widgets::secondary_button(ui, tl!("Open the DeviantArt developer page"), 260.0).clicked() {
        crate::links::open(app, ui.ctx(), DEVELOPER_PAGE);
    }
    ui.add_space(8.0);
    ui.label(tl!("DeviantArt app id"));
    let mut client_id = f.get("clientId").and_then(Value::as_str).unwrap_or("").to_string();
    if ui.add(egui::TextEdit::singleline(&mut client_id).desired_width(ui.available_width())).changed() {
        f.insert("clientId".into(), json!(client_id.clone()));
    }
    ui.label(egui::RichText::new(tl!("The DeviantArt login stays in the system keychain, not in PhotoCraft's settings file.")).color(t.text_dim));
    ui.add_space(4.0);
    let connected = app.deviantart.ready();
    if connected {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(tl!("Connected")).strong());
            if crate::widgets::secondary_button(ui, tl!("Connect again"), 150.0).clicked() {
                app.deviantart.client_id = client_id.clone();
                if let Err(error) = start_connect(app, "") {
                    app.ui.status = error;
                    app.ui.status_error = true;
                }
            }
            ui.add_enabled_ui(app.deviantart_job.is_none(), |ui| {
                if crate::widgets::secondary_button(ui, tl!("Disconnect DeviantArt"), 170.0).clicked() {
                    errors::disconnect(app);
                }
            });
        });
    } else if crate::widgets::secondary_button(ui, tl!("Connect DeviantArt"), 220.0).clicked() {
        app.deviantart.client_id = client_id;
        if let Err(error) = start_connect(app, "") {
            app.ui.status = error;
            app.ui.status_error = true;
        }
    }
    ui.add_space(8.0);
    ui.label(tl!("Title"));
    let mut title = f.get("title").and_then(Value::as_str).unwrap_or("").to_string();
    if ui.add(egui::TextEdit::singleline(&mut title).desired_width(ui.available_width())).changed() {
        f.insert("title".into(), json!(title));
    }
    ui.label(tl!("Tags"));
    let mut tags = f.get("tags").and_then(Value::as_str).unwrap_or("").to_string();
    if ui.add(egui::TextEdit::singleline(&mut tags).desired_width(ui.available_width())).changed() {
        f.insert("tags".into(), json!(tags));
    }
    ui.label(egui::RichText::new(tl!("Letters, numbers and underscores, separated by commas.")).color(t.text_dim));
    ui.add_space(8.0);
    ui.label(tl!("Submit as"));
    let format = f.get("format").and_then(Value::as_str).unwrap_or("psd").to_string();
    ui.horizontal(|ui| {
        for (id, label) in [("file", tl!("Saved file")), ("psd", "PSD"), ("png", "PNG")] {
            if ui.selectable_label(format == id, label).clicked() {
                f.insert("format".into(), json!(id));
            }
        }
    });
    ui.label(egui::RichText::new(tl!("PNG is what DeviantArt shows. A PSD is only stored as a file in Sta.sh.")).color(t.text_dim));
    ui.label(egui::RichText::new(tl!("OK uploads to Sta.sh. Nothing is public until you publish.")).color(t.text_dim));
    ui.add_space(8.0);
    gallery_section(app, ui, f);
    ui.add_space(8.0);
    let mut more = f.get("more").and_then(Value::as_bool).unwrap_or(false);
    if crate::widgets::checkbox(ui, &mut more, tl!("More options")).changed() {
        f.insert("more".into(), json!(more));
    }
    if !more {
        return;
    }
    ui.add_space(4.0);
    ui.label(egui::RichText::new(tl!("Connect again after this update so gallery, publish and the profile picture are allowed.")).color(t.text_dim));
    ui.label(tl!("Sta.sh item id"));
    let mut item_id = f.get("itemId").and_then(Value::as_str).unwrap_or("").to_string();
    if item_id.is_empty() {
        item_id = app.deviantart.last_item_id.clone();
    }
    if ui.add(egui::TextEdit::singleline(&mut item_id).desired_width(120.0)).changed() {
        f.insert("itemId".into(), json!(item_id.clone()));
    }
    let mut public_ok = f.get("publicOk").and_then(Value::as_bool).unwrap_or(false);
    if crate::widgets::checkbox(ui, &mut public_ok, tl!("Publish this publicly")).changed() {
        f.insert("publicOk".into(), json!(public_ok));
    }
    let mut mature = f.get("mature").and_then(Value::as_bool).unwrap_or(false);
    if crate::widgets::checkbox(ui, &mut mature, tl!("Not for all ages")).changed() {
        f.insert("mature".into(), json!(mature));
    }
    ui.label(tl!("License"));
    let mut creative_commons = f.get("creativeCommons").and_then(Value::as_bool).unwrap_or(false);
    if crate::widgets::checkbox(ui, &mut creative_commons, tl!("Creative Commons")).changed() {
        f.insert("creativeCommons".into(), json!(creative_commons));
    }
    let mut commercial = f.get("commercial").and_then(Value::as_bool).unwrap_or(false);
    if crate::widgets::checkbox(ui, &mut commercial, tl!("Commercial use")).changed() {
        f.insert("commercial".into(), json!(commercial));
    }
    if crate::widgets::secondary_button(ui, tl!("Publish"), 160.0).clicked() {
        if !public_ok {
            app.ui.status = tl!("Confirm that this publish is public.").to_string();
            app.ui.status_error = true;
        } else if item_id.trim().is_empty() || item_id == "0" {
            app.ui.status = tl!("Upload with OK first. Publish uses the Sta.sh item from that upload.").to_string();
            app.ui.status_error = true;
        } else if !app.deviantart.scopes.split_whitespace().any(|scope| scope == "publish") {
            app.ui.status = tl!("Reconnect to DeviantArt to grant permission to publish.").to_string();
            if let Err(error) = start_connect(app, "publish") {
                app.ui.status = error;
                app.ui.status_error = true;
            }
        } else if let Err(error) = start_call(
            app,
            &json!({"op":"publish","itemId": item_id.parse::<i64>().unwrap_or(0), "mature": mature, "level": "moderate", "classes": [], "creativeCommons": creative_commons, "commercial": commercial, "modify": "no"}).to_string(),
        ) {
            app.ui.status = error;
            app.ui.status_error = true;
        }
    }
    ui.label(tl!("Deviation id"));
    let mut deviation_id = f.get("deviationId").and_then(Value::as_str).unwrap_or("").to_string();
    if ui.add(egui::TextEdit::singleline(&mut deviation_id).desired_width(ui.available_width())).changed() {
        f.insert("deviationId".into(), json!(deviation_id.clone()));
    }
    if crate::widgets::secondary_button(ui, tl!("Update title and tags"), 220.0).clicked() {
        let tags = clean_tags(f.get("tags").and_then(Value::as_str).unwrap_or(""));
        let title = f.get("title").and_then(Value::as_str).unwrap_or("");
        if let Err(error) = start_call(app, &json!({"op":"edit","deviationId": deviation_id, "title": title, "tags": tags}).to_string()) {
            app.ui.status = error;
            app.ui.status_error = true;
        }
    }
    ui.label(tl!("Profile picture"));
    let avatar = f.get("avatar").and_then(Value::as_str).unwrap_or("png").to_string();
    ui.horizontal(|ui| {
        for (id, label) in [("png", "PNG"), ("jpg", "JPEG")] {
            if ui.selectable_label(avatar == id, label).clicked() {
                f.insert("avatar".into(), json!(id));
            }
        }
    });
    if crate::widgets::secondary_button(ui, tl!("Set as DeviantArt profile picture"), 260.0).clicked() {
        if !app.deviantart.scopes.split_whitespace().any(|scope| scope == "user.manage") {
            app.ui.status = tl!("Reconnect to DeviantArt to grant permission to change your profile picture.").to_string();
            if let Err(error) = start_connect(app, "user.manage") {
                app.ui.status = error;
                app.ui.status_error = true;
            }
            return;
        }
        match avatar_file(app, &avatar) {
            Ok(path) => {
                if let Err(error) = start_call(app, &json!({"op":"avatar","path": path}).to_string()) {
                    app.ui.status = error;
                    app.ui.status_error = true;
                }
            }
            Err(error) => {
                app.ui.status = error;
                app.ui.status_error = true;
            }
        }
    }
    ui.add_space(8.0);
    crate::widgets::section_label(ui, "Submit automatically");
    ui.add_space(4.0);
    for (key, label) in [
        ("onSave", tl!("Also submit when I Save")),
        ("onSaveAs", tl!("Also submit when I Save As")),
        ("onSaveCopy", tl!("Also submit when I Save a Copy")),
        ("onExport", tl!("Also submit when I Export")),
    ] {
        let mut on = f.get(key).and_then(Value::as_bool).unwrap_or(false);
        if crate::widgets::checkbox(ui, &mut on, label).changed() {
            f.insert(key.into(), json!(on));
        }
    }
}
