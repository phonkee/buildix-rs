#![allow(dead_code)]
#![allow(unused_imports)]

use buildix::sort::Sort;
use buildix_derive::{Filter, Select, SelectBuilder};

#[allow(unused_imports)]
use buildix::prelude::*;

#[test]
fn test_offset_limit() {
    let mut query = OffsetLimitBuilder::default();

    let (q, _v) = query.get_query();
    assert_eq!(q, r#"SELECT id FROM user"#);

    query.limit = Some(42);
    let (q, _v) = query.get_query();
    assert_eq!(q, r#"SELECT id FROM user LIMIT 42"#);

    query.offset = 84;

    let (q, _v) = query.get_query();
    assert_eq!(q, r#"SELECT id FROM user LIMIT 42 OFFSET 84"#);
}

#[derive(Default, SelectBuilder)]
struct OffsetLimitBuilder {
    #[buildix(select)]
    select: Vec<SelectUser>,

    #[buildix(offset)]
    offset: i32,

    #[buildix(limit)]
    limit: Option<i32>,
}

#[derive(Default, Select)]
#[buildix(from(table(name = "user")))]
struct SelectUser {
    id: i32,
}
