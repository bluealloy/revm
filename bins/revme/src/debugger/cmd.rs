

pub enum Control {
    Exit,
    Step,
    Continue,
    Breakpoint,
    InsertAccount,
    // RewindCall,
    // RewindOpcode,
    // Stack,
    // StackSet,
    // Memory,
    // MemorySet,
    // Account,
    // AccountSetBalance,
    // AccountSetNonce,
    // Storage,
    // StorageSet,
}

impl Control {
    pub fn parse(cmd: &[&str]) -> Result<Control, String> {
        let mut len = cmd.len();
        let mut check = |pop: usize, err: &str| -> Result<(), String> {
            if pop > len {
                return Err(err.into());
            }
            len -= pop;
            Ok(())
        };
        check(1, "Command not found")?;
        let cmd = match cmd[0] {
            "exit" => Control::Exit,
            "step" => Control::Step,
            "continue" => Control::Continue,
            "breakpoint" => Control::Breakpoint,
            "insert" => Control::InsertAccount,
            t => return Err(format!("Command {:?} not found",t)),
        };

        Ok(cmd)
    }
}
