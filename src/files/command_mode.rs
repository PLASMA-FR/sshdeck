#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileCommand {
    Cd(String),
    Mkdir(String),
    Touch(String),
    Rename(String, String),
    DownloadSelected,
    Upload(String),
    Chmod(String, String),
    Chown(String, String),
    Open,
    Edit,
    CopyPath,
    BookmarkAdd(String),
    BookmarkJump(String),
    Unknown(String),
}

pub fn parse_command(input: &str) -> FileCommand {
    let command = input.strip_prefix(':').unwrap_or(input).trim();
    let parts = match shell_words::split(command) {
        Ok(parts) => parts,
        Err(_) => return FileCommand::Unknown(input.into()),
    };

    match parts.as_slice() {
        [cmd, path] if cmd == "cd" => FileCommand::Cd(path.clone()),
        [cmd, path] if cmd == "mkdir" => FileCommand::Mkdir(path.clone()),
        [cmd, path] if cmd == "touch" => FileCommand::Touch(path.clone()),
        [cmd, old, new] if cmd == "rename" || cmd == "mv" => {
            FileCommand::Rename(old.clone(), new.clone())
        }
        [cmd, sel] if cmd == "download" && sel == "selected" => FileCommand::DownloadSelected,
        [cmd, path] if cmd == "upload" => FileCommand::Upload(path.clone()),
        [cmd, mode, path] if cmd == "chmod" => FileCommand::Chmod(mode.clone(), path.clone()),
        [cmd, owner, path] if cmd == "chown" => FileCommand::Chown(owner.clone(), path.clone()),
        [cmd] if cmd == "open" => FileCommand::Open,
        [cmd] if cmd == "edit" => FileCommand::Edit,
        [cmd] if cmd == "copy-path" => FileCommand::CopyPath,
        [a, b, c] if a == "bookmark" && b == "add" => FileCommand::BookmarkAdd(c.clone()),
        [a, b, c] if a == "bookmark" && b == "jump" => FileCommand::BookmarkJump(c.clone()),
        _ => FileCommand::Unknown(input.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_basic_file_commands() {
        assert_eq!(parse_command(":cd /var/www"), FileCommand::Cd("/var/www".into()));
        assert_eq!(
            parse_command("mkdir '~/new folder'"),
            FileCommand::Mkdir("~/new folder".into())
        );
        assert_eq!(parse_command(":open"), FileCommand::Open);
        assert_eq!(parse_command(":edit"), FileCommand::Edit);
    }

    #[test]
    fn parses_mutation_commands_with_quoted_paths() {
        assert_eq!(
            parse_command(":rename 'old name.txt' 'new name.txt'"),
            FileCommand::Rename("old name.txt".into(), "new name.txt".into())
        );
        assert_eq!(
            parse_command(":chmod 0644 '/tmp/a b'"),
            FileCommand::Chmod("0644".into(), "/tmp/a b".into())
        );
        assert_eq!(
            parse_command(":chown deploy:www-data /var/www"),
            FileCommand::Chown("deploy:www-data".into(), "/var/www".into())
        );
    }

    #[test]
    fn parses_transfer_and_bookmark_commands() {
        assert_eq!(
            parse_command(":download selected"),
            FileCommand::DownloadSelected
        );
        assert_eq!(
            parse_command(":upload './release build.tar.gz'"),
            FileCommand::Upload("./release build.tar.gz".into())
        );
        assert_eq!(
            parse_command(":bookmark jump webroot"),
            FileCommand::BookmarkJump("webroot".into())
        );
    }

    #[test]
    fn malformed_or_extra_args_are_unknown() {
        assert_eq!(
            parse_command(":cd '/unterminated"),
            FileCommand::Unknown(":cd '/unterminated".into())
        );
        assert_eq!(
            parse_command(":touch a b"),
            FileCommand::Unknown(":touch a b".into())
        );
        assert_eq!(
            parse_command(":download /tmp/archive.tar.gz"),
            FileCommand::Unknown(":download /tmp/archive.tar.gz".into())
        );
    }
}
