pub enum Command {
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

impl Command {
    pub fn parse(cmd: &[&str]) -> Result<Command, String> {
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
            "exit" => Command::Exit,
            "step" => Command::Step,
            "continue" => Command::Continue,
            "breakpoint" => Command::Breakpoint,
            "insert" => Command::InsertAccount,
            t => return Err(format!("Command {:?} not found",t)),
        };

        Ok(cmd)
    }
}
