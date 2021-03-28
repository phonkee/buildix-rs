# Buildix

Query builder for sqlx. Buildix takes different approach to query building.
Buildix provides a set of derive macros to define query builder structures.
You can easily define query builders with large possibilities.
Real power comes in filter, where you can define multiple filtering capabilities
and then set only some filters. Buildix is aware of Option values and
generates efficient code.
Buildix shines when you have e.g. api that supports filtering by multiple
fields. Usual query builders are defined where used, on the other hand
when you define buildix builder once, you can use it multiple places.

Please refer to example builder to see how buildix will work.

##### Warning

This project is work in progress and is heavily developed. First implementation
will implement "general" sql (POC), and then I will implement implementations
for Postgres, MySQL, sqlite - databases that are supported in sqlx.
This project is unusable in this phase and the api can change.
Do not hack buildix with implementing its traits, because that's internal
functionality and will be subject to change.
Only use derive macros to work with it.

#### Features:

Features that I am working on (in order)

- SelectBuilder
  - [x] Base query
  - [x] Join
  - [x] Sort
  - [x] Limit
  - [x] Offset
  - [x] Group
  - [x] Filter (Implemented - testing)
  - [ ] Count
  - [ ] Having
  - [ ] Map - callback support
  - [ ] Execute
  - [ ] Stream support (low priority)
  - [ ] support all dialects (Postgres, MySQL, SQLite, MS SQL) - (design)
- DeleteBuilder
  - [x] Filter (shared with SelectBuilder)
  - [x] Limit (shared with SelectBuilder)
  - [ ] Count
  - [x] Map - callback support
  - [ ] Execute
- InsertBuilder
  - [ ] Insert
  - [ ] On duplicate key
  - [ ] Returning auto_increment
  - [ ] Map - callback support
  - [ ] Execute w. Batch support
- UpdateBuilder
  - [ ] Update
  - [ ] Filter
  - [ ] Count
  - [ ] Map - callback support
  - [ ] Execute w. Batch support

# Example select query builder

```rust
#[derive(Default, SelectBuilder)]
struct SelectUserBuilder {
    #[buildix(select)]
    select: Vec<SelectUser>,

    #[buildix(filter)]
    filter: Filter,

    #[buildix(limit)]
    limit: Option<i32>,

    #[buildix(offset)]
    offset: i32,

    #[buildix(sort = "name")]
    sort_name: Option<buildix::sort::Sort>,

    #[buildix(sort = "age")]
    sort_age: buildix::sort::Sort,

    #[buildix(count)]
    count: i32,
}

#[derive(Default, Select)]
#[buildix(from(table(name = "user", alias = "u")))]
#[buildix(from(join(name = "order", alias = "o", on = "o.user_id = u.id")))]
#[buildix(group = "name", group = "email")]
struct SelectUser {
    #[buildix(table = "u")]
    name: String,

    #[buildix(table = "u")]
    email: String,

    #[buildix(table = "u")]
    age: Option<i64>,

    #[buildix(expr = "IF(age > 18, true, false)")]
    is_adult: bool,

    #[buildix(expr = r#"COALESCE(other, "")"#)]
    other: String,
}

#[derive(Default, Filter)]
#[buildix(operator = "OR")]
struct Filter {
    author_id: Option<i32>,

    #[buildix(expr = "last_updated < ?")]
    last_updated: Option<i32>,

    // automatically provides filter = "priority = ?"
    priority: i32,

    #[buildix(expr = "age > ?", isnull)]
    age: Option<i32>,

    // inner filter will be sub clause in parentheses (if needed)
    inner: InnerFilter,

    // inner filter will be sub clause in parentheses (if needed) 
    inner: InnerFilter,
    
    // Vec automatically converts to IN(...), if no value is available
    // this filter will not be available in where clause
    id: Vec<i32>,
}

// even multiple filters supported
#[derive(Default, Filter)]
#[buildix(operator = "AND")]
struct InnerFilter {
    value: Option<i32>,
    value2: Option<i32>,
}

```

# Filter

Buildix filter is powerful. Buildix helps you create filters that will be
written to sql only if they are set, it supports null and more.
It is very simple to filter by multiple filter values, and even embedded
filters which translate into sub clauses. This gives you powerful tool
for querying. After you define query once and verify query field names,
you can reuse it safely.

Lets have a look at previous example how it actually translates to sql
query.

Lets try to see what previous example looks like. You can see it in
tests directory [test_readme_select.rs](buildix/tests/test_readme_select.rs)


First lets create default querybuilder.

```rust
let mut qb = SelectUserBuilder::default();
let (q, _) = query.to_sql::<Postgres>().unwrap();
```

query is now

```sql
SELECT u.name, u.email, u.age, IF(age > 18, true, false) AS is_adult, COALESCE(other, "") AS other FROM user AS u, INNER JOIN order o (o.user_id = u.id) WHERE (priority = ? OR age ISNULL) GROUP BY name, email ORDER BY age ASC
```

if we set inner filter value

```rust
let mut qb = SelectUserBuilder::default();
let (q, _) = query.to_sql::<Postgres>().unwrap();
qb.filter.inner.value = Some(42);
qb.filter.inner.value2 = Some(314);
```

now query is

```sql
SELECT u.name, u.email, u.age, IF(age > 18, true, false) AS is_adult, COALESCE(other, "") AS other FROM user AS u, INNER JOIN order o (o.user_id = u.id) WHERE (priority = ? OR age ISNULL OR (value = ? AND value2 = ?)) GROUP BY name, email ORDER BY age ASC
```

You can see how powerful this filtering is. Not to say that there is more
functionality that helps you to build reliable query builders.


# Execute

When Buildix executes query, it will fill the values directly on the structure,
that's why you need to always work with builders as `&mut`.
That's real power since you don't need to see underlying implementation
you will only get results directly instantiated as you provided.

Buildix will support selecting single instance or vec, and in future also
Stream.


# Delete query builder


```rust
#[derive(DeleteBuilder)]
#[buildix(table="user", map="map_delete")]
struct UserDeleteBuilder {
    #[buildix(filter)]
    filter: Filter,

    #[buildix(limit)]
    limit: Option<i32>,

    #[buildix(count)]
    count: i32, // if configured, buildix will populate how many records has been deleted
}

// map_delete checks delete query (e.g. if limit is present)
pub fn map_delete(builder: &mut UserDeleteBuilder) -> buildix::Result<()> {
    if builder.filter.user_id == 0 {
        // return error or set user_id to Option<i32>
    }
    Ok(())
}

#[derive(FilterQuery)]
#[buildix(operator = "OR")]
struct Filter {
    #[buildix(filter = "age > ?", isnull)]
    age: Option<i32>,
    
    #[buildix(filter = "user_id = ?")]
    user_id: i32,
}
```

# Insert query builder

```rust
#[derive(InsertBuilder)]
struct UserInsertBuilder {
    #[buildix(insert)]
    insert: Vec<InsertUser>,

    #[buildix(count)]
    count: i32,
}

#[derive(Insert)]
#[buildix(table = "user", unique_key = "id")]
struct InsertUser {
    #[buildix(update)]
    name: String,

    // this field will not be updated when doing `ON DUPLICATE KEY`
    email: String,

    #[buildix(update)]
    age: Option<i64>,
    
    // #[buildix(returning)]
    // id: i32,
}
```

# Update query builder

```rust
#[derive(UpdateBuilder)]
struct UserUpdateBuilder {
    #[buildix(update)]
    update: Vec<Update>,
    
    #[buildix(count)]
    count: i32,
}

#[derive(Update)]
#[buildix(table = "user")]
struct UpdateUser {
    name: String,
    email: String,
    age: Option<i64>,
    
    #[buildix(filter)]
    filter: UpdateFilter,
}

#[derive(Default, Filter)]
pub struct UpdateFilter {
    id: i32,
}

```


# Author

Peter Vrba <phonkee@pm.me>