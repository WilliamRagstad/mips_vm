//! Debugger command handling

use crate::debugger::Debugger;

pub enum Command {
    Step,
    Continue,
    Breakpoint(u32),
    RemoveBreakpoint(u32),
    Print(String),
    Quit,
}

impl Command {
    pub fn parse(input: &str) -> Option<Command> {
        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        
        match parts.get(0) {
            Some(&"s") | Some(&"step") => Some(Command::Step),
            Some(&"c") | Some(&"continue") => Some(Command::Continue),
            Some(&"b") | Some(&"break") => {
                if let Some(addr_str) = parts.get(1) {
                    if let Ok(addr) = u32::from_str_radix(addr_str.trim_start_matches("0x"), 16) {
                        return Some(Command::Breakpoint(addr));
                    }
                }
                None
            }
            Some(&"d") | Some(&"delete") => {
                if let Some(addr_str) = parts.get(1) {
                    if let Ok(addr) = u32::from_str_radix(addr_str.trim_start_matches("0x"), 16) {
                        return Some(Command::RemoveBreakpoint(addr));
                    }
                }
                None
            }
            Some(&"p") | Some(&"print") => {
                if let Some(expr) = parts.get(1) {
                    return Some(Command::Print(expr.to_string()));
                }
                None
            }
            Some(&"q") | Some(&"quit") => Some(Command::Quit),
            _ => None,
        }
    }

    pub fn execute(&self, debugger: &mut Debugger) -> Result<String, Box<dyn std::error::Error>> {
        match self {
            Command::Step => {
                debugger.step()?;
                Ok("Stepped one instruction".to_string())
            }
            Command::Continue => {
                debugger.run()?;
                Ok("Continued execution".to_string())
            }
            Command::Breakpoint(addr) => {
                debugger.add_breakpoint(*addr);
                Ok(format!("Breakpoint added at 0x{:08x}", addr))
            }
            Command::RemoveBreakpoint(addr) => {
                debugger.remove_breakpoint(*addr);
                Ok(format!("Breakpoint removed at 0x{:08x}", addr))
            }
            Command::Print(expr) => {
                // TODO: Implement expression evaluation
                Ok(format!("Print: {}", expr))
            }
            Command::Quit => Ok("Goodbye!".to_string()),
        }
    }
}
