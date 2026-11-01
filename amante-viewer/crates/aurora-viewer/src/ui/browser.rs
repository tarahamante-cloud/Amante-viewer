//! A standalone in-world browser window backed by the viewer's CEF/Chromium
//! media plugin. Its private profile is kept separate from grid-authenticated
//! pages such as Search and Marketplace.

use super::web_view::WebView;
use super::widgets::{self, Floater};
use aurora_media::{BrowserSettings, MediaPlugin, PluginPaths};
use egui::Vec2;

pub const WINDOW: &str = "amante_web_browser";

pub struct BrowserUi {
    pub open: bool,
    address: String,
    address_error: Option<String>,
    web: WebView,
}

impl Default for BrowserUi {
    fn default() -> Self {
        Self {
            open: false,
            address: String::new(),
            address_error: None,
            web: WebView::private(crate::settings::cache_dir().join(crate::media::private_profile::ROOT)),
        }
    }
}

impl BrowserUi {
    pub fn toggle(&mut self) {
        self.open = !self.open;
        if self.open {
            self.web.retry();
        } else if self.web.running() {
            self.web.close();
        }
    }

    /// The browser page that has keyboard focus, for forwarding native input.
    pub fn focused_web(&mut self) -> Option<&mut MediaPlugin> {
        (self.open && self.web.focused).then(|| self.web.plugin_mut()).flatten()
    }

    pub fn show(&mut self, ctx: &egui::Context, p: &crate::theme::Palette, paths: Option<&PluginPaths>, browser: &BrowserSettings) {
        if !self.open {
            return;
        }

        let work = super::layout::work_area(ctx);
        let size = Vec2::new(940.0, 640.0).min(work.size());
        let pos = (work.center() - size * 0.5).max(work.min);
        let mut window = Floater::new(WINDOW, "Amante Web Browser", pos, size);
        window.min_size = Vec2::new(600.0, 360.0).min(size);
        let mut editing_address = false;
        window.show(ctx, p, &mut self.open, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Back").clicked() {
                    self.web.back();
                }
                if ui.button("Forward").clicked() {
                    self.web.forward();
                }
                if ui.button("Reload").clicked() {
                    self.web.reload();
                }
                let address = ui.add(
                    egui::TextEdit::singleline(&mut self.address)
                        .id(egui::Id::unique("browser_address"))
                        .hint_text("Enter a website address (https:// or http://)")
                        .desired_width(f32::INFINITY),
                );
                editing_address = address.has_focus();
                let enter = address.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                if ui.button("Go").clicked() || enter {
                    match browser_url(&self.address) {
                        Ok(url) => {
                            self.address = url.clone();
                            self.address_error = None;
                            self.web.navigate_again(&url);
                        }
                        Err(message) => {
                            self.address_error = Some(message.to_owned());
                        }
                    }
                }
            });
            if let Some(error) = &self.address_error {
                ui.colored_label(p.danger, error);
            }
            widgets::divider(ui, p);

            if self.web.loading {
                ui.label("Loading page…");
            }
            let size = ui.available_size().max(Vec2::splat(100.0));
            if self.address.is_empty() && !self.web.running() {
                ui.allocate_ui_with_layout(size, egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.add_space((size.y * 0.35).max(20.0));
                    ui.label(
                        egui::RichText::new("Browse the web inside Amante Viewer")
                            .size(18.0)
                            .strong()
                            .color(p.ink),
                    );
                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new("Enter a website address above. Pages use the viewer's Chromium media plugin.").color(p.muted),
                    );
                });
            } else {
                self.web.show(ui, p, paths, browser, None, size);
                if !editing_address && !self.web.location.is_empty() {
                    // Keep the address in step with navigation from links.
                    self.address.clone_from(&self.web.location);
                }
            }
        });

        if !self.open && self.web.running() {
            self.web.close();
        }
    }
}

fn browser_url(address: &str) -> Result<String, &'static str> {
    let address = address.trim();
    if address.is_empty() {
        return Err("Enter a website address");
    }
    let lower = address.to_ascii_lowercase();
    if lower.starts_with("https://") || lower.starts_with("http://") {
        return Ok(address.to_owned());
    }
    if address.contains("://") || lower.starts_with("file:") || lower.starts_with("javascript:") || lower.starts_with("data:") {
        return Err("Only http:// and https:// addresses are supported");
    }
    Ok(format!("https://{address}"))
}

#[cfg(test)]
mod tests {
    use super::browser_url;

    #[test]
    fn browser_accepts_http_and_adds_https_to_hostnames() {
        assert_eq!(browser_url("https://example.com").unwrap(), "https://example.com");
        assert_eq!(browser_url("http://example.com/path").unwrap(), "http://example.com/path");
        assert_eq!(browser_url("example.com").unwrap(), "https://example.com");
    }

    #[test]
    fn browser_rejects_non_web_schemes() {
        assert!(browser_url("file:///tmp/index.html").is_err());
        assert!(browser_url("javascript:alert(1)").is_err());
    }
}
