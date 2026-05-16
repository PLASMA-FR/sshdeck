use ratatui::{prelude::*, widgets::*};

use crate::{app::App, mouse::ClickTarget, widgets::{button, button::ButtonKind}};

/// First-run onboarding card for the empty-host state. The dashboard uses the
/// same actions inline; this view exists so onboarding can become a dedicated
/// flow without scattering copy or click targets.
pub fn draw(f: &mut Frame, app: &mut App, area: Rect) {
    let card = crate::design::layout::centered(area, 58, 12);
    let add = ClickTarget::ModalButton("add-host".into());
    let import = ClickTarget::ModalButton("import-hosts".into());
    app.mouse.register(Rect { x: card.x + 10, y: card.y + 6, width: 14, height: 1 }, add.clone());
    app.mouse.register(Rect { x: card.x + 27, y: card.y + 6, width: 22, height: 1 }, import.clone());

    let lines = vec![
        Line::from(Span::styled("No SSH hosts yet", app.theme.title())),
        Line::raw(""),
        Line::from("Add your first server or import from ~/.ssh/config."),
        Line::from(Span::styled("SSHDeck keeps managed hosts local and asks before touching config.", app.theme.muted())),
        Line::raw(""),
        Line::from(vec![button::label(app, "Add Host", &add, ButtonKind::Primary), Span::raw("  "), button::label(app, "Import SSH Config", &import, ButtonKind::Secondary)]),
    ];

    f.render_widget(
        Paragraph::new(lines).alignment(Alignment::Center).style(app.theme.surface()).block(
            Block::bordered()
                .border_type(crate::design::borders::rounded(!app.ascii))
                .border_style(app.theme.border())
                .title(" SSHDeck "),
        ),
        card,
    );
}
