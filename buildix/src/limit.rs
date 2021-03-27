use sqlx::Database;

pub trait Limit {
    fn get_limit<DB: Database>(self) -> Option<String>;
}

macro_rules! impl_limit {
    ($T:ty) => {
        impl Limit for $T {
            fn get_limit<DB: Database>(self) -> Option<String> {
                if self < 0 {
                    None
                } else {
                    Some(format!("LIMIT {}", self))
                }
            }
        }
    };
}

impl_limit!(i32);
impl_limit!(i64);

impl<T> Limit for Option<T>
where
    T: Limit,
{
    fn get_limit<DB: Database>(self) -> Option<String> {
        match self {
            Some(t) => t.get_limit::<DB>(),
            None => None,
        }
    }
}
