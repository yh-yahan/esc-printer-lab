use std::ops::Range;

use crate::parser::command::Command;
use crate::receipt::receipt::Receipt;

#[derive(Clone, Debug)]
pub struct ParsedCommand {
    pub command: Command,
    pub span: Range<usize>,
}

pub struct PrintSession {
    pub raw: Vec<u8>,
    pub commands: Vec<ParsedCommand>,
    pub receipts: Vec<Receipt>,
    pub current: Receipt,
    pub epoch: u64,
}

impl PrintSession {
    pub fn new() -> Self {
        Self {
            raw: Vec::new(),
            commands: Vec::new(),
            receipts: Vec::new(),
            current: Receipt::new(),
            epoch: 0,
        }
    }

    pub fn push_raw(&mut self, data: &[u8]) {
        self.raw.extend_from_slice(data);
    }

    pub fn push_command(&mut self, command: ParsedCommand) {
        self.commands.push(command);
    }

    pub fn update_current(&mut self, receipt: Receipt) {
        self.current = receipt;
    }

    pub fn commit_current(&mut self) {
        if self.current.items.is_empty() {
            return;
        }

        let done = std::mem::take(&mut self.current);
        self.receipts.push(done);
    }

    pub fn ticket_count(&self) -> usize {
        self.receipts.len() + usize::from(!self.current.items.is_empty())
    }

    pub fn clear(&mut self) {
        self.raw.clear();
        self.commands.clear();
        self.receipts.clear();
        self.current = Receipt::new();
        self.epoch = self.epoch.wrapping_add(1);
    }
}
