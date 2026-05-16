use ratatui::{prelude::*, widgets::*};
use crate::{app::{App, Mode}, views, widgets::{modal, toast}};

pub fn draw(f:&mut Frame, app:&mut App){
    app.mouse.begin_frame();
    let area=f.area();
    if area.width < crate::design::layout::MIN_WIDTH || area.height < crate::design::layout::MIN_HEIGHT {
        let p=Paragraph::new("SSHDeck needs a larger terminal window.\nMinimum recommended size: 100x30.\n\nTip: mouse and keyboard controls are both supported.").alignment(Alignment::Center).block(Block::bordered().border_type(crate::design::borders::rounded(!app.ascii)).title(" ▣ SSHDeck "));
        f.render_widget(p, area); return;
    }
    match app.view { crate::app::View::Dashboard=>views::dashboard::draw(f, app, area), crate::app::View::HostDetail=>views::host_detail::draw(f, app, area), crate::app::View::Files=>views::files::draw(f, app, area), crate::app::View::Tunnels=>views::tunnels::draw(f, app, area), crate::app::View::CommandRunner=>views::command_runner::draw(f, app, area), crate::app::View::Logs=>views::logs::draw(f, app, area), crate::app::View::Settings=>views::settings::draw(f, app, area), crate::app::View::Help=>views::help::draw(f, app, area) }
    if app.mode==Mode::Palette { modal::command_palette(f, app, area); }
    if app.mode==Mode::Search { modal::search(f, app, area); }
    if app.mode==Mode::Command { modal::command_mode(f, app, area); }
    if app.context_menu.is_some() { modal::context_menu(f, app, area); }
    if let Some(t)=&app.toast { toast::draw(f, app, t, area); }
}
