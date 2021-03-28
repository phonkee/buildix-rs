use sqlx::Database;

pub trait Offset {
    fn get_offset<DB: Database>(self) -> Option<String>;
}

macro_rules! impl_offset {
    ($T:ty) => {
        impl Offset for $T {
            fn get_offset<DB: Database>(self) -> Option<String> {
                if self <= 0 {
                    None
                } else {
                    Some(format!("OFFSET {}", self))
                }
            }
        }
    };
}

impl_offset!(i32);
impl_offset!(i64);

impl<T> Offset for Option<T>
where
    T: Offset,
{
    fn get_offset<DB: Database>(self) -> Option<String> {
        match self {
            Some(t) => t.get_offset::<DB>(),
            None => None,
        }
    }
}
