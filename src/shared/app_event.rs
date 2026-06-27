use crate::receipt::receipt::Receipt;

pub enum AppEvent {
    ReceiptParsed(Receipt),
}
