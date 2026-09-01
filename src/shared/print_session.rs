use crate::parser::command::Command;
use crate::receipt::receipt::Receipt;

#[derive(Default)]
pub struct PrintSession {
    pub raw: Vec<u8>,
    pub commands: Vec<Command>,
    pub receipts: Vec<Receipt>,
}

impl PrintSession {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_raw(&mut self, data: &[u8]) {
        self.raw.extend_from_slice(data);
    }

    pub fn push_command(&mut self, command: Command) {
        self.commands.push(command);
    }

    pub fn push_receipt(&mut self, receipt: Receipt) {
        self.receipts.push(receipt);
    }

    pub fn clear(&mut self) {
        self.raw.clear();
        self.commands.clear();
        self.receipts.clear();
    }
}
