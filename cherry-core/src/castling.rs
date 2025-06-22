use crate::File;

#[derive(Debug, Copy, Clone)]
pub struct CastlingRights {
    pub short: Option<File>,
    pub long: Option<File>,
}