use ratatui::widgets::BorderType; pub fn rounded(unicode:bool)->BorderType{ if unicode{BorderType::Rounded}else{BorderType::Plain} }
