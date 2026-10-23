//! Community and project links: the Help menu, the About dialog, the start screen and the title
//! bar all open these. URLs follow the crafting-app pattern (`getartcraft.com/apps/<app>`,
//! `github.com/storytold/<app>`).
//!
//! Under `harmonyos-ui`, project/community URLs are omitted from the Help menu. The privacy
//! statement opens the AppGallery-hosted agreement page, and the issue tracker is this app's
//! own public report repository.

use serde_json::{Value, json};

use crate::PhotocraftApp;

/// Whether this host exposes community and project entry points.
pub const SHOW_PROJECT_LINKS: bool = !cfg!(feature = "harmonyos-ui");

#[cfg(not(feature = "harmonyos-ui"))]
pub const DISCORD: &str = "https://discord.gg/artcraft";
#[cfg(feature = "harmonyos-ui")]
pub const DISCORD: &str = "";

pub const ARTCRAFT_WEBSITE: &str = "https://getartcraft.com";
pub const APP_PAGE: &str = "https://getartcraft.com/apps/photocraft";

#[cfg(not(feature = "harmonyos-ui"))]
pub const GITHUB: &str = "https://github.com/storytold/photocraft";
#[cfg(not(feature = "harmonyos-ui"))]
pub const ISSUES: &str = "https://github.com/storytold/photocraft/issues";

/// HarmonyOS: the app's remotely hosted privacy statement (Help › 隐私政策).
#[cfg(feature = "harmonyos-ui")]
pub const PRIVACY_STATEMENT: &str = "https://www.a999.cloud/statics/artforge-photo-privacy-policy.html";

/// HarmonyOS: the public issue tracker for this app (Help › 问题反馈).
#[cfg(feature = "harmonyos-ui")]
pub const ISSUES: &str = "https://gitcode.com/MXJinjing/artforge-photo-report";

/// Upstream PhotoCraft (desktop / Web) open-source repository.
pub const PHOTOCRAFT_GITHUB: &str = "https://github.com/storytold/photocraft";
/// HarmonyOS native open-source port that this listing fork is based on.
pub const PHOTOCRAFT_HMOS_GITHUB: &str = "https://github.com/MXJinjing/photocraft-hmos";

/// Help-menu link commands: (id, url). Labels live in `menus::UI_COMMANDS`.
#[cfg(not(feature = "harmonyos-ui"))]
pub const COMMANDS: &[(&str, &str)] =
    &[("help.discord", DISCORD), ("help.website", APP_PAGE), ("help.artcraftWebsite", ARTCRAFT_WEBSITE), ("help.github", GITHUB), ("help.reportIssue", ISSUES)];

#[cfg(feature = "harmonyos-ui")]
pub const COMMANDS: &[(&str, &str)] = &[("help.privacyPolicy", PRIVACY_STATEMENT), ("help.reportIssue", ISSUES)];

pub fn url_for(id: &str) -> Option<&'static str> {
    COMMANDS.iter().find(|c| c.0 == id).map(|c| c.1)
}

/// Open `url` in the system browser (a new tab on the web) and note it in the status bar. Prefers
/// the platform `open_url` service (reliable on Windows/macOS/Linux); falls back to `ctx.open_url`.
pub fn open(app: &mut PhotocraftApp, ctx: &egui::Context, url: &str) -> Value {
    let opened = app.services.open_url.as_ref().map(|f| f(url).is_ok()).unwrap_or(false);
    if !opened {
        ctx.open_url(egui::OpenUrl::new_tab(url));
    }
    let template = "Opened {url}";
    app.ui.status = crate::i18n::fmt(tl!(template), &[("url", url)]);
    json!({"url": url})
}

/// The prominent "Join us on Discord" button.
pub fn discord_button(app: &mut PhotocraftApp, ui: &mut egui::Ui, min_width: f32) -> egui::Response {
    #[cfg(not(feature = "harmonyos-ui"))]
    {
        let r = crate::widgets::primary_button(ui, tl!("Join us on Discord"), min_width).on_hover_text(DISCORD);
        if r.clicked() {
            open(app, ui.ctx(), DISCORD);
        }
        r
    }
    #[cfg(feature = "harmonyos-ui")]
    {
        let _ = (app, min_width);
        // Project links are hidden on HarmonyOS; keep a no-op stub for shared call sites.
        ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover())
    }
}

/// "PhotoCraft website · GitHub · ArtCraft" as links, centred. Clicks route through [`open`] (the
/// platform browser service) rather than `ui.hyperlink_to`, which uses the unreliable `ctx.open_url`.
pub fn link_row(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    #[cfg(feature = "harmonyos-ui")]
    {
        let _ = (app, ui);
    }
    #[cfg(not(feature = "harmonyos-ui"))]
    {
        let t = crate::theme::Tokens::get(ui.ctx());
        let links = [(tl!("PhotoCraft website"), APP_PAGE), (tl!("GitHub"), GITHUB), (tl!("ArtCraft"), ARTCRAFT_WEBSITE)];
        let font = egui::FontId::proportional(12.5);
        let sep = "  ·  ";
        let width: f32 = links.iter().map(|(l, _)| ui.painter().layout_no_wrap((*l).into(), font.clone(), t.text).size().x).sum::<f32>()
            + 2.0 * ui.painter().layout_no_wrap(sep.into(), font.clone(), t.text).size().x;
        let mut clicked: Option<&str> = None;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.add_space(((ui.available_width() - width) / 2.0).max(0.0));
            for (i, (label, url)) in links.iter().enumerate() {
                if i > 0 {
                    ui.label(egui::RichText::new(sep).font(font.clone()).color(t.text_faint));
                }
                if ui.link(egui::RichText::new(*label).font(font.clone()).color(t.accent)).on_hover_text(*url).clicked() {
                    clicked = Some(url);
                }
            }
        });
        if let Some(url) = clicked {
            open(app, ui.ctx(), url);
        }
    }
}

/// HarmonyOS About: one line of open-source attribution with inline GitHub links (not in Help).
#[cfg(feature = "harmonyos-ui")]
pub fn about_oss_attribution(app: &mut PhotocraftApp, ui: &mut egui::Ui) {
    let zh = crate::brand::ui_is_chinese();
    let (pre, mid, post) = if zh { ("基于开源项目 ", " 与 ", "（MIT / Apache-2.0）") } else { ("Based on open-source ", " and ", " (MIT / Apache-2.0)") };
    ui.add_space(8.0);
    let mut clicked = None;
    let mut show_licenses = false;
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label(pre);
        if ui.link("PhotoCraft").on_hover_text(PHOTOCRAFT_GITHUB).clicked() {
            clicked = Some(PHOTOCRAFT_GITHUB);
        }
        ui.label(mid);
        if ui.link("photocraft-hmos").on_hover_text(PHOTOCRAFT_HMOS_GITHUB).clicked() {
            clicked = Some(PHOTOCRAFT_HMOS_GITHUB);
        }
        ui.label(post);
        let license_label = if zh { "（查看许可证）" } else { " (View licenses)" };
        if ui.link(egui::RichText::new(license_label).color(crate::theme::Tokens::get(ui.ctx()).accent)).clicked() {
            show_licenses = true;
        }
    });
    if let Some(url) = clicked {
        open(app, ui.ctx(), url);
    }
    if show_licenses {
        app.ui.open_dialog(crate::state::DialogKind::About, serde_json::from_value(json!({"licenseInfo": true})).unwrap_or_default());
    }
}

/// License texts shipped in the native HarmonyOS application.
#[cfg(feature = "harmonyos-ui")]
pub(crate) const LICENSE_MIT: &str = include_str!("../../../LICENSE-MIT");
#[cfg(feature = "harmonyos-ui")]
pub(crate) const LICENSE_APACHE: &str = include_str!("../../../LICENSE-APACHE");
#[cfg(feature = "harmonyos-ui")]
pub(crate) const LICENSE_NOTICE: &str = include_str!("../../../NOTICE");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_open_through_the_platform_service() {
        // Issue #14: links (Help menu, Discord button, start-page links) must open in the browser.
        // They route through the `open_url` service rather than the unreliable `ctx.open_url`.
        use std::sync::{Arc, Mutex};
        let opened: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let rec = opened.clone();
        let services = crate::Services {
            open_url: Some(Box::new(move |u: &str| {
                rec.lock().unwrap().push(u.to_string());
                Ok(())
            })),
            ..Default::default()
        };
        let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), services);
        let ctx = egui::Context::default();
        // Every Help-menu link id reaches the service with its URL.
        for (id, url) in COMMANDS {
            crate::menus::invoke(&mut app, &ctx, id, json!({})).unwrap();
            assert_eq!(opened.lock().unwrap().last().map(String::as_str), Some(*url), "{id}");
        }
        #[cfg(not(feature = "harmonyos-ui"))]
        {
            // The direct open() helper (Discord button / start-page links) also uses it.
            open(&mut app, &ctx, DISCORD);
            assert_eq!(opened.lock().unwrap().last().map(String::as_str), Some(DISCORD));
        }
    }

    #[test]
    fn help_restore_submenu_restores_builtin_swatches() {
        let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), Default::default());
        let ids: Vec<String> =
            crate::menus::menu_items(&app).into_iter().filter(|i| i.path == ["Help", "Restore Defaults"]).map(|i| i.id).collect();
        assert_eq!(ids, [
            "help.restore.brushes",
            "help.restore.swatches",
            "help.restore.patterns",
            "help.restore.gradients",
            "help.restore.styles",
            "help.restore.shapes",
            "help.restore.toolPresets"
        ]);
        app.session.execute("swatches.delete", json!({"swatch": "Black"})).unwrap();
        crate::menus::invoke(&mut app, &egui::Context::default(), "help.restore.swatches", json!({})).unwrap();
        assert!(app.session.presets.swatches.iter().any(|g| g.items.iter().any(|s| s.name == "Black")));
    }

    #[cfg(not(feature = "harmonyos-ui"))]
    #[test]
    fn help_menu_lists_links_then_separator_then_system_info_and_about() {
        let app = PhotocraftApp::new(photocraft_engine::Session::new(), Default::default());
        let help: Vec<String> = crate::menus::menu_items(&app).into_iter().filter(|i| i.path == ["Help"]).map(|i| i.id).collect();
        if cfg!(any(debug_assertions, feature = "stylus-debug-menu")) {
            assert_eq!(help, ["help.discord", "help.website", "help.artcraftWebsite", "help.github", "help.reportIssue", "---", "help.systemInfo", "help.stylusDebug", "help.about"]);
        } else {
            assert_eq!(help, ["help.discord", "help.website", "help.artcraftWebsite", "help.github", "help.reportIssue", "---", "help.systemInfo", "help.about"]);
        }
        for (id, _) in COMMANDS {
            assert!(crate::menus::is_live(id) && crate::menus::is_enabled(&app, id), "{id}");
        }
    }

    #[cfg(feature = "harmonyos-ui")]
    #[test]
    fn harmonyos_help_menu_lists_privacy_link_issue_tracker_and_local_information() {
        let app = PhotocraftApp::new(photocraft_engine::Session::new(), Default::default());
        let help: Vec<String> = crate::menus::menu_items(&app).into_iter().filter(|i| i.path == ["Help"]).map(|i| i.id).collect();
        if cfg!(any(debug_assertions, feature = "stylus-debug-menu")) {
            assert_eq!(help, ["help.privacyPolicy", "help.reportIssue", "---", "help.systemInfo", "help.stylusDebug", "help.about"]);
        } else {
            assert_eq!(help, ["help.privacyPolicy", "help.reportIssue", "---", "help.systemInfo", "help.about"]);
        }
        let links: Vec<&str> = COMMANDS.iter().map(|c| c.0).collect();
        assert_eq!(links, ["help.privacyPolicy", "help.reportIssue"]);
        assert_eq!(url_for("help.privacyPolicy"), Some(PRIVACY_STATEMENT));
        assert_eq!(url_for("help.reportIssue"), Some(ISSUES));
        // No user agreement: the app has no account system and no networking.
        assert!(url_for("help.userAgreement").is_none());
        assert!(url_for("help.github").is_none());
        assert!(url_for("help.discord").is_none());
        assert!(!DISCORD.contains("discord"));
    }

    #[cfg(feature = "harmonyos-ui")]
    #[test]
    fn harmonyos_home_hides_promo_links_and_about_shows_oss_github() {
        use egui_kittest::{Harness, kittest::Queryable};
        use std::sync::{Arc, Mutex};
        let opened: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let rec = opened.clone();
        let services = crate::Services {
            open_url: Some(Box::new(move |u: &str| {
                rec.lock().unwrap().push(u.to_string());
                Ok(())
            })),
            ..Default::default()
        };
        let app = PhotocraftApp::new(photocraft_engine::Session::new(), services);
        let mut h = Harness::builder().with_size(egui::vec2(1400.0, 900.0)).build_ui_state(
            |ui, app| {
                app.host_logic(ui.ctx());
                app.host_ui(ui);
            },
            app,
        );
        PhotocraftApp::setup_context(&h.ctx, crate::theme::ThemeKind::ALL[0]);
        h.run_steps(3);
        for label in ["Discord", "Join us on Discord", "PhotoCraft website", "ArtCraft"] {
            assert!(h.query_by_label(label).is_none(), "home: {label}");
        }
        // Help must not list GitHub / Discord; About is the OSS credit surface.
        let help: Vec<String> = crate::menus::menu_items(h.state()).into_iter().filter(|i| i.path == ["Help"]).map(|i| i.id).collect();
        assert!(!help.iter().any(|id| id == "help.github" || id == "help.discord"));

        for tab in ["about", "contributors", "models"] {
            h.state_mut().ui.dialogs.clear();
            h.state_mut().ui.open_dialog(crate::state::DialogKind::About, serde_json::from_value(json!({"tab": tab})).unwrap());
            h.run_steps(3);
            assert!(h.query_by_label(&format!("About {}", crate::brand::APP_NAME)).is_some());
            assert!(h.query_by_label(crate::brand::app_name()).is_some(), "About heading shows the product name");
            assert!(crate::brand::mark_rect(&h.ctx).is_some(), "About paints the brand icon");
            assert!(h.query_by_label_contains("版本号: ").is_some(), "current HAP version is shown");
            assert!(h.query_by_label_contains("open-source").is_some() || h.query_by_label_contains("开源项目").is_some());
            assert!(h.query_by_label_contains("MXJinjing").is_none(), "no maintainer blurb");
            assert!(h.query_by_label("PhotoCraft").is_some());
            assert!(h.query_by_label("photocraft-hmos").is_some());
            for label in ["Contributors", "Models", "Join us on Discord", "PhotoCraft website", "ArtCraft"] {
                assert!(h.query_by_label(label).is_none(), "about {tab}: {label}");
            }
        }
        h.get_by_label("PhotoCraft").click();
        h.run_steps(1);
        assert_eq!(opened.lock().unwrap().last().map(String::as_str), Some(PHOTOCRAFT_GITHUB));
        h.get_by_label("photocraft-hmos").click();
        h.run_steps(1);
        assert_eq!(opened.lock().unwrap().last().map(String::as_str), Some(PHOTOCRAFT_HMOS_GITHUB));
    }

    #[test]
    fn link_commands_open_their_urls() {
        let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), Default::default());
        let ctx = egui::Context::default();
        for (id, url) in COMMANDS {
            let r = crate::menus::invoke(&mut app, &ctx, id, serde_json::json!({})).unwrap();
            assert_eq!(r["url"], *url);
            assert_eq!(app.ui.status, format!("Opened {url}"));
        }
    }
}
