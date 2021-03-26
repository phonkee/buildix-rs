pub trait Offset {
    fn get_offset(self) -> Option<String>;
}

impl Offset for i32 {
    fn get_offset(self) -> Option<String> {
        if self <= 0 {
            None
        } else {
            Some(format!("OFFSET {}", self))
        }
    }
}

impl Offset for i64 {
    fn get_offset(self) -> Option<String> {
        if self <= 0 {
            None
        } else {
            Some(format!("OFFSET {}", self))
        }
    }
}

impl<T> Offset for Option<T>
where
    T: Offset,
{
    fn get_offset(self) -> Option<String> {
        match self {
            Some(t) => t.get_offset(),
            None => None,
        }
    }
}
