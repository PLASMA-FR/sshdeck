#[derive(Debug, Clone, Copy)] pub struct Icons{pub server:&'static str,pub terminal:&'static str,pub files:&'static str,pub tunnel:&'static str,pub logs:&'static str,pub all:&'static str,pub folder:&'static str,pub file:&'static str,pub lock:&'static str,pub favorite:&'static str}
pub fn nerd()->Icons{ Icons{server:"󰣀",terminal:"",files:"󰉋",tunnel:"󰩠",logs:"󰗼",all:"󰈞",folder:"󰉋",file:"󰈙",lock:"",favorite:"★"} }
pub fn ascii()->Icons{ Icons{server:"[S]",terminal:">$",files:"[D]",tunnel:"<>",logs:"log",all:"*",folder:"[D]",file:"[F]",lock:"!",favorite:"*"} }
