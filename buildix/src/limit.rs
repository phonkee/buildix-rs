pub trait Limit {
    fn get_limit(self) -> Option<String>;
}

impl Limit for i32 {
    fn get_limit(self) -> Option<String> {
        if self < 0 {
            None
        } else {
            Some(format!("LIMIT {}", self))
        }
    }
}

impl Limit for i64 {
    fn get_limit(self) -> Option<String> {
        if self < 0 {
            None
        } else {
            Some(format!("LIMIT {}", self))
        }
    }
}

impl<T> Limit for Option<T>
where
    T: Limit,
{
    fn get_limit(self) -> Option<String> {
        match self {
            Some(t) => t.get_limit(),
            None => None,
        }
    }
}
