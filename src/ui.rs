use ratatui::{prelude::*, widgets::*};
use crate::{app::{App, Mode}, views, widgets::{logo, modal, toast}};

pub fn draw(f:&mut Frame, app:&mut App){
    app.mouse.begin_frame();
    let area=f.area();
    if area.width < crate::design::layout::MIN_WIDTH || area.height < crate::design::layout::MIN_HEIGHT {
        let p=Paragraph::new("SSHDeck needs a larger terminal window.\nMinimum recommended size: 100x30.\n\nTip: mouse and keyboard controls are both supported.").alignment(Alignment::Center).block(Block::bordered().border_type(crate::design::borders::rounded(!app.ascii)).title(" ▣ SSHDeck "));
        f.render_widget(p, area); return;
    }
    if app.show_splash() {
        let outer = Layout::vertical([Constraint::Fill(1), Constraint::Length(29), Constraint::Fill(1)]).split(area);
        let width = if area.width > 96 { 86 } else { area.width.saturating_sub(4) };
        let inner = Layout::horizontal([Constraint::Fill(1), Constraint::Length(width), Constraint::Fill(1)]).split(outer[1]);
        f.render_widget(logo::splash(app), inner[1]);
        return;
    }
    match app.view { crate::app::View::Dashboard=>views::dashboard::draw(f, app, area), crate::app::View::HostDetail=>views::host_detail::draw(f, app, area), crate::app::View::Files=>views::files::draw(f, app, area), crate::app::View::Tunnels=>views::tunnels::draw(f, app, area), crate::app::View::CommandRunner=>views::command_runner::draw(f, app, area), crate::app::View::Logs=>views::logs::draw(f, app, area), crate::app::View::Settings=>views::settings::draw(f, app, area), crate::app::View::Help=>views::help::draw(f, app, area) }
    if app.mode==Mode::Palette { modal::command_palette(f, app, area); }
    if app.mode==Mode::Search { modal::search(f, app, area); }
    if app.mode==Mode::Command { modal::command_mode(f, app, area); }
    if app.mode==Mode::HostForm { modal::host_form(f, app, area); }
    if app.mode==Mode::Confirm { modal::confirm(f, app, area); }
    if app.context_menu.is_some() { modal::context_menu(f, app, area); }
    if let Some(t)=&app.toast { toast::draw(f, app, t, area); }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};
    use unicode_width::UnicodeWidthStr;

    use crate::{
        app::{AppOptions, ContextMenu, HostFormMode, HostFormState, Mode, Toast, ToastLevel, View},
        config::{app_config::AppConfig, managed_hosts::HostDraft},
        files::file_entry::{FileEntry, FileKind},
        mouse::ClickTarget,
        ssh::host::SshHost,
        widgets::status_bar,
    };

    fn test_app() -> App {
        let mut app = App::new(
            AppConfig::default(),
            AppOptions { no_animations: true, ascii: true, mouse: true },
        ).unwrap();
        app.hosts = vec![
            SshHost { alias: "alpha".into(), hostname: Some("alpha.local".into()), user: Some("root".into()), group: Some("Lab".into()), ..Default::default() },
            SshHost { alias: "beta".into(), hostname: Some("beta.local".into()), user: Some("admin".into()), group: Some("Prod".into()), ..Default::default() },
        ];
        app.filtered = vec![0, 1];
        app.selected = 0;
        app.remote_entries = vec![
            FileEntry { name: "src".into(), path: "~/src".into(), kind: FileKind::Directory, size: 0, permissions: "drwxr-xr-x".into(), modified: "May 16 09:00".into(), owner: "demo".into(), group: "demo".into(), selected: false },
            FileEntry { name: "README.md".into(), path: "~/README.md".into(), kind: FileKind::File, size: 2048, permissions: "-rw-r--r--".into(), modified: "May 16 09:01".into(), owner: "demo".into(), group: "demo".into(), selected: false },
            FileEntry { name: "logs".into(), path: "~/logs".into(), kind: FileKind::Directory, size: 0, permissions: "drwxr-xr-x".into(), modified: "May 16 09:02".into(), owner: "demo".into(), group: "demo".into(), selected: false },
            FileEntry { name: "app.conf".into(), path: "~/app.conf".into(), kind: FileKind::File, size: 512, permissions: "-rw-r--r--".into(), modified: "May 16 09:03".into(), owner: "demo".into(), group: "demo".into(), selected: false },
        ];
        app.toast = None;
        app
    }


    fn first_rect_for(app: &App, target: &ClickTarget) -> Option<ratatui::prelude::Rect> {
        app.mouse.registry.regions().iter().find(|r| &r.target == target).map(|r| r.rect)
    }

    fn assert_center_hits(app: &App, target: ClickTarget) {
        let rect = first_rect_for(app, &target).unwrap_or_else(|| panic!("missing region for {:?}", target));
        let x = rect.x + rect.width.saturating_sub(1) / 2;
        let y = rect.y + rect.height.saturating_sub(1) / 2;
        assert_eq!(app.mouse.registry.hit(x, y), Some(target));
    }

    fn render_at(app: &mut App, width: u16, height: u16) -> String {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| draw(f, app)).unwrap();
        buffer_text(terminal.backend().buffer())
    }

    fn render(app: &mut App) -> String {
        render_at(app, 120, 40)
    }

    fn buffer_text(buffer: &ratatui::buffer::Buffer) -> String {
        let mut text = String::new();
        for cell in buffer.content() {
            text.push_str(cell.symbol());
        }
        text
    }

    fn assert_regions_within(app: &App, width: u16, height: u16) {
        for region in app.mouse.registry.regions() {
            let right = u32::from(region.rect.x) + u32::from(region.rect.width);
            let bottom = u32::from(region.rect.y) + u32::from(region.rect.height);
            assert!(
                right <= u32::from(width) && bottom <= u32::from(height),
                "region {:?} exceeds {}x{}",
                region,
                width,
                height
            );
        }
    }

    #[test]
    fn dashboard_mouse_regions_match_rendered_rows_and_buttons() {
        let mut app = test_app();
        render(&mut app);

        assert_eq!(app.mouse.registry.hit(1, 4), Some(ClickTarget::SidebarGroup("All".into())));
        assert_eq!(app.mouse.registry.hit(19, 5), Some(ClickTarget::HostRow(0)));
        assert_eq!(app.mouse.registry.hit(19, 6), Some(ClickTarget::HostRow(1)));
        assert_center_hits(&app, ClickTarget::HostConnectButton(0));
        assert_center_hits(&app, ClickTarget::HostEditButton(0));
        assert_center_hits(&app, ClickTarget::ModalButton("add-host".into()));
    }

    #[test]
    fn host_form_mouse_regions_match_fields_and_buttons() {
        let mut app = test_app();
        app.host_form = Some(HostFormState { mode: HostFormMode::Add, draft: HostDraft::default(), field: 0, messages: Vec::new(), test_result: None, original_alias: None });
        app.mode = Mode::HostForm;
        render(&mut app);

        assert_eq!(app.mouse.registry.hit(48, 10), Some(ClickTarget::FormField("alias".into())));
        assert_eq!(app.mouse.registry.hit(48, 11), Some(ClickTarget::FormField("hostname/ip".into())));
        assert_center_hits(&app, ClickTarget::ModalButton("test-host".into()));
        assert_center_hits(&app, ClickTarget::ModalButton("save-host".into()));
        assert_center_hits(&app, ClickTarget::ModalButton("cancel".into()));
    }

    #[test]
    fn files_mouse_regions_match_list_rows() {
        let mut app = test_app();
        app.view = View::Files;
        render(&mut app);

        assert!(matches!(app.mouse.registry.hit(2, 4), Some(ClickTarget::Breadcrumb(_))));
        assert!(matches!(app.mouse.registry.hit(31, 4), Some(ClickTarget::FileEntry(_))));
        assert!(matches!(app.mouse.registry.hit(31, 7), Some(ClickTarget::FileEntry(_))));
    }

    #[test]
    fn tunnel_mouse_regions_match_choices_and_action_buttons() {
        let mut app = test_app();
        app.view = View::Tunnels;
        render(&mut app);

        assert_eq!(app.mouse.registry.hit(3, 1), Some(ClickTarget::TunnelType("local".into())));
        assert_eq!(app.mouse.registry.hit(3, 2), Some(ClickTarget::TunnelType("remote".into())));
        assert_center_hits(&app, ClickTarget::ModalButton("start-tunnel".into()));
        assert_center_hits(&app, ClickTarget::ModalButton("copy-tunnel".into()));
    }

    #[test]
    fn command_palette_mouse_regions_match_action_rows() {
        let mut app = test_app();
        app.mode = Mode::Palette;
        render(&mut app);

        assert_eq!(app.mouse.registry.hit(33, 15), Some(ClickTarget::CommandPaletteItem("Add host".into())));
        assert_eq!(app.mouse.registry.hit(33, 16), Some(ClickTarget::CommandPaletteItem("Open files".into())));
    }

    #[test]
    fn settings_mouse_regions_match_preference_rows() {
        let mut app = test_app();
        app.view = View::Settings;
        render(&mut app);

        assert_eq!(app.mouse.registry.hit(2, 4), Some(ClickTarget::SettingRow("theme".into())));
        assert_eq!(app.mouse.registry.hit(2, 5), Some(ClickTarget::SettingRow("animations".into())));
        assert_eq!(app.mouse.registry.hit(2, 9), Some(ClickTarget::SettingRow("show_hidden".into())));
    }

    #[test]
    fn all_views_render_at_minimum_and_standard_bounds() {
        let cases = [
            (View::Dashboard, "Hosts"),
            (View::HostDetail, "SSHDeck Host Detail"),
            (View::Files, "Files"),
            (View::Tunnels, "Tunnel Builder"),
            (View::CommandRunner, "Remote Command Runner"),
            (View::Logs, "SSHDeck Events"),
            (View::Settings, "Settings"),
            (View::Help, "SSHDeck Help"),
        ];

        for (view, needle) in cases {
            for (width, height) in [(100, 30), (120, 40)] {
                let mut app = test_app();
                app.view = view.clone();
                app.logs = vec!["alpha connected".into(), "beta disconnected".into()];
                app.command_output = "uptime\nload average: 0.01".into();

                let frame = render_at(&mut app, width, height);

                assert!(frame.contains(needle), "{view:?} did not render {needle:?} at {width}x{height}");
                assert_regions_within(&app, width, height);
            }
        }
    }

    #[test]
    fn too_small_terminal_uses_guard_view_and_clears_mouse_regions() {
        for (width, height) in [(99, 30), (100, 29), (80, 24)] {
            let mut app = test_app();
            app.view = View::Files;
            app.mode = Mode::Palette;

            let frame = render_at(&mut app, width, height);

            assert!(frame.contains("SSHDeck needs a larger terminal window"));
            assert!(app.mouse.registry.regions().is_empty());
        }

        let mut tiny = test_app();
        render_at(&mut tiny, 1, 1);
        assert!(tiny.mouse.registry.regions().is_empty());
    }

    #[test]
    fn overlay_modes_render_with_regions_inside_bounds() {
        let overlay_cases: [(&str, fn(&mut App), &str); 6] = [
            ("palette", |app| app.mode = Mode::Palette, "Command palette"),
            ("search", |app| { app.mode = Mode::Search; app.search = "alp".into(); }, "Search"),
            ("command", |app| { app.mode = Mode::Command; app.command_input = "uptime".into(); }, "Command"),
            ("host form", |app| {
                app.mode = Mode::HostForm;
                app.host_form = Some(HostFormState { mode: HostFormMode::Add, draft: HostDraft::default(), field: 0, messages: Vec::new(), test_result: None, original_alias: None });
            }, "Add Host"),
            ("context menu", |app| {
                app.context_menu = Some(ContextMenu { title: "Host Actions".into(), items: vec![("Connect".into(), ClickTarget::HostConnectButton(0))] });
            }, "Host Actions"),
            ("toast", |app| {
                app.toast = Some(Toast { message: "Saved host".into(), ttl: 5, level: ToastLevel::Success });
            }, "Saved host"),
        ];

        for (name, setup, needle) in overlay_cases {
            let mut app = test_app();
            setup(&mut app);

            let frame = render_at(&mut app, 120, 40);

            assert!(frame.contains(needle), "{name} overlay did not render {needle:?}");
            assert_regions_within(&app, 120, 40);
        }
    }

    #[test]
    fn splash_screen_is_skippable_and_timer_based() {
        let mut app = App::new(
            AppConfig::default(),
            AppOptions { no_animations: false, ascii: true, mouse: true },
        ).unwrap();
        assert!(app.show_splash());
        render(&mut app);
        assert!(app.mouse.registry.regions().is_empty());
        app.on_tick();
        assert!(app.show_splash());
        app.dismiss_splash();
        assert!(!app.show_splash());
    }

    #[test]
    fn status_bar_mouse_regions_match_visible_shortcut_chips() {
        let mut app = test_app();
        render(&mut app);

        let status_y = 39;
        let prefix = format!(" dashboard  {} servers · mouse on  ", app.hosts.len());
        let first_x = UnicodeWidthStr::width(prefix.as_str()) as u16;

        let search = first_rect_for(&app, &ClickTarget::StatusShortcut("/".into())).unwrap();
        assert_eq!(search, Rect { x: first_x, y: status_y, width: UnicodeWidthStr::width("  / search  ") as u16, height: 1 });
        assert_eq!(app.mouse.registry.hit(first_x, status_y), Some(ClickTarget::StatusShortcut("/".into())));
        assert_eq!(app.mouse.registry.hit(first_x + search.width - 1, status_y), Some(ClickTarget::StatusShortcut("/".into())));

        let add = first_rect_for(&app, &ClickTarget::StatusShortcut("a".into())).unwrap();
        assert_eq!(add.x, search.x + search.width + 1);
        assert_eq!(add.y, status_y);
        assert_eq!(app.mouse.registry.hit(add.x, status_y), Some(ClickTarget::StatusShortcut("a".into())));
        assert_eq!(app.mouse.registry.hit(add.x + add.width - 1, status_y), Some(ClickTarget::StatusShortcut("a".into())));

        let help = first_rect_for(&app, &ClickTarget::StatusShortcut("?".into())).unwrap();
        assert_eq!(help.y, status_y);
        assert_eq!(app.mouse.registry.hit(help.x + help.width / 2, status_y), Some(ClickTarget::StatusShortcut("?".into())));
    }

    #[test]
    fn status_bar_preserves_help_chip_when_shortcuts_overflow() {
        let mut app = test_app();
        let backend = TestBackend::new(44, 1);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.draw(|f| {
            app.mouse.begin_frame();
            status_bar::draw(
                f,
                &mut app,
                Rect { x: 0, y: 0, width: 44, height: 1 },
                "/ search · a add · Enter connect · s files · t tunnel · h health · r command · ? help",
            );
        }).unwrap();

        let help = first_rect_for(&app, &ClickTarget::StatusShortcut("?".into())).unwrap();
        assert_eq!(app.mouse.registry.hit(help.x + help.width / 2, 0), Some(ClickTarget::StatusShortcut("?".into())));
    }
}
