use std::process::Command;

/// Task trait abstraction
pub trait Task: Send + Sync {
    fn execute(&self) -> Result<(), String>;
}

/// Blanket implementation so existing fn() -> Result<(), String> still works.
impl<F> Task for F
where
    F: Fn() -> Result<(), String> + Send + Sync,
{
    fn execute(&self) -> Result<(), String> {
        (self)()
    }
}

/// Shell task implementation
pub struct ShellTask {
    pub command: String,
}

impl ShellTask {
    pub fn new<S: Into<String>>(command: S) -> Self {
        ShellTask {
            command: command.into(),
        }
    }
}

impl Task for ShellTask {
    fn execute(&self) -> Result<(), String> {
        let status = Command::new("/bin/sh")
            .arg("-c")
            .arg(&self.command)
            .status()
            .map_err(|e| format!("Command failed: {}", e))?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("Command failed with status: {}", status))
        }
    }
}

/// Python task implementation
pub struct PythonTask {
    pub code: String,
    pub interpreter: String,
}

impl PythonTask {
    pub fn new<S: Into<String>>(code: S) -> Self {
        PythonTask {
            code: code.into(),
            interpreter: "python3".into(),
        }
    }

    pub fn with_interpreter<C: Into<String>, I: Into<String>>(code: C, interpreter: I) -> Self {
        PythonTask {
            code: code.into(),
            interpreter: interpreter.into(),
        }
    }
}

impl Task for PythonTask {
    fn execute(&self) -> Result<(), String> {
        let status = Command::new(&self.interpreter)
            .arg("-c")
            .arg(&self.code)
            .status()
            .map_err(|e| format!("Command failed: {}", e))?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("Command failed with status: {}", status))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    fn has_python3() -> bool {
        Command::new("python3").arg("--version").status().is_ok()
    }

    #[test]
    fn fn_task_ok_executes() {
        fn ok() -> Result<(), String> {
            Ok(())
        }
        let result = Task::execute(&ok);
        assert!(result.is_ok());
    }

    #[test]
    fn fn_task_err_executes() {
        fn err() -> Result<(), String> {
            Err("boom".into())
        }
        let result = Task::execute(&err);
        assert!(result.is_err());
    }

    #[test]
    fn shell_task_ok() {
        // 'true' is a standard POSIX command that exits 0
        let t = ShellTask::new("true");
        assert!(t.execute().is_ok());
    }

    #[test]
    fn shell_task_err() {
        // 'false' is a standard POSIX command that exits non-zero
        let t = ShellTask::new("false");
        assert!(t.execute().is_err());
    }

    #[test]
    fn python_task_ok_if_available() {
        if !has_python3() {
            eprintln!("python3 not available; skipping test");
            return;
        }
        let t = PythonTask::new("print('hi')");
        assert!(t.execute().is_ok());
    }

    #[test]
    fn python_task_err_if_available() {
        if !has_python3() {
            eprintln!("python3 not available; skipping test");
            return;
        }
        let t = PythonTask::new("import sys; sys.exit(2)");
        assert!(t.execute().is_err());
    }

    #[test]
    fn python_task_custom_interpreter_ok_if_available() {
        if !has_python3() {
            eprintln!("python3 not available; skipping test");
            return;
        }
        let t = PythonTask::with_interpreter("print('custom')", "python3");
        assert!(t.execute().is_ok());
    }
}
