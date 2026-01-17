#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Opcode {
    Set = 0x01,
    Get = 0x02,
    Del = 0x03,
    Ping = 0x04,
    Stats = 0x05,
    Keys = 0x06,      // List all keys (with optional prefix filter)
    SetConfig = 0x07, // Dynamic config update (e.g., memory limit)

    Subscribe = 0x20,
    Unsubscribe = 0x21,

    PushEvent = 0x80,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Status {
    Ok = 0x00,
    NotFound = 0x01,

    ErrBadPayload = 0x10,
    ErrUnsupportedFormat = 0x11,
    ErrTooLarge = 0x12,
    ErrInternal = 0x13,
    ErrUnauthorized = 0x14,
    ErrLagged = 0x15,
    ErrInvalidKeyFormat = 0x16,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ValueFormat {
    Json = 1,
    MsgPack = 2,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum EventType {
    Invalidate = 1,
    TableChanged = 2, // implicit upsert
}

impl From<u8> for Status {
    fn from(v: u8) -> Self {
        match v {
            0x00 => Status::Ok,
            0x01 => Status::NotFound,
            0x10 => Status::ErrBadPayload,
            0x11 => Status::ErrUnsupportedFormat,
            0x12 => Status::ErrTooLarge,
            0x13 => Status::ErrInternal,
            0x14 => Status::ErrUnauthorized,
            0x15 => Status::ErrLagged,
            0x16 => Status::ErrInvalidKeyFormat,
            _ => Status::ErrInternal,
        }
    }
}
