use crate::receipt::receipt::Receipt;

#[derive(Default)]
pub struct PrintSession {
    pub raw_chunks: Vec<Vec<u8>>,
    pub parser_output: Vec<String>,
    pub escpos_output: Vec<String>,
    pub receipts: Vec<Receipt>,
}

impl PrintSession {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_raw(&mut self, data: &[u8]) {
        self.raw_chunks.push(data.to_vec());
    }

    pub fn push_parser(&mut self, msg: impl Into<String>) {
        self.parser_output.push(msg.into());
    }

    pub fn push_receipt(&mut self, receipt: Receipt) {
        self.receipts.push(receipt);
    }

    pub fn clear(&mut self) {
        self.escpos_output.clear();
        self.raw_chunks.clear();
        self.parser_output.clear();
        self.receipts.clear();
    }
}
