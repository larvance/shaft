use std::path::Path;
use std::process::Command;

pub fn reveal_in_terminal<P: AsRef<Path>>(path: P) -> Result<(), String> {
    let p = path.as_ref();
    let dir = if p.is_dir() {
        p.to_path_buf()
    } else {
        p.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| Path::new(".").to_path_buf())
    };

    #[cfg(target_os = "linux")]
    {
        let terminals = ["x-terminal-emulator", "gnome-terminal", "alacritty", "kitty", "konsole", "xfce4-terminal", "xterm"];
        for term in terminals {
            if let Ok(child) = Command::new(term).current_dir(&dir).spawn() {
                // Detach and let it run
                let _ = child.id();
                return Ok(());
            }
        }
        return Err("No supported terminal emulator found in system PATH".into());
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg("-a")
            .arg("Terminal")
            .arg(&dir)
            .spawn()
            .map_err(|e| format!("Failed to open terminal on macOS: {}", e))?;
        return Ok(());
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("cmd.exe")
            .arg("/K")
            .current_dir(&dir)
            .spawn()
            .map_err(|e| format!("Failed to open terminal on Windows: {}", e))?;
        return Ok(());
    }

    #[allow(unreachable_code)]
    Ok(())
}

pub fn copy_file<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> Result<u64, String> {
    let s = src.as_ref();
    let d = dst.as_ref();

    if !s.exists() {
        return Err(format!("Source file does not exist: {}", s.display()));
    }

    if let Some(parent) = d.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
    }

    std::fs::copy(s, d).map_err(|e| format!("Failed to copy file: {}", e))
}

pub fn move_file<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> Result<(), String> {
    let s = src.as_ref();
    let d = dst.as_ref();

    if !s.exists() {
        return Err(format!("Source file does not exist: {}", s.display()));
    }

    if let Some(parent) = d.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
    }

    std::fs::rename(s, d).map_err(|e| format!("Failed to move file: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_and_move_file() {
        let temp = std::env::temp_dir();
        let src = temp.join("shaft_action_src.txt");
        let copy_dst = temp.join("shaft_action_copy.txt");
        let move_dst = temp.join("shaft_action_moved.txt");

        std::fs::write(&src, "Action payload").unwrap();

        assert!(copy_file(&src, &copy_dst).is_ok());
        assert!(copy_dst.exists());

        assert!(move_file(&copy_dst, &move_dst).is_ok());
        assert!(!copy_dst.exists());
        assert!(move_dst.exists());

        let _ = std::fs::remove_file(&src);
        let _ = std::fs::remove_file(&move_dst);
    }
}
