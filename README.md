# Buildix

Declarative query builder for sqlx.  

Buildix takes different approach to query building. Instead of defining
queries inplace, buildix provides set of derive macros, that generate
effective code for building and running sql queries.
You can easily define query builders with large possibilities.
Real power comes in filter, where you can define multiple filtering capabilities
and then only set some filters needed.
Buildix is aware of Option values and can omit values that are not set.
This brings power to any apis that need to filter on multiple columns
with ease.

Please refer to example select builder to see how buildix will work.

##### Warning

This project is work in progress and is heavily developed.
Currently buildix does not return any arguments, nor it's executing queries.
I am working hard to bring all the functionalities, so please be kind to me.
Design is kinda done, but there will be definitely some changes.
Buildix currently generates some form of general sql for select and delete.
Please stay tuned and star this repository.

First implementation will implement "general" sql (POC), and then I will work on
implementations  for Postgres, MySQL, sqlite - databases that are supported in sqlx.

This project is currently unusable in this phase and the api can change.
Do not hack buildix with implementing its traits, because that's internal
functionality and will be subject to change.

Only use derive macros to work with it.

The vision is to bring at least basic power in span of next weeks.

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
  - [x] Map - callback support
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

    #[buildix(table = "u", column = "custom_age")]
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
    
    // Vec automatically converts to IN(...), if no value is available
    // this filter will not be available in where clause
    // if you want to set expr. on this field, just provide single
    // ? and buildix will expand it automatically
    // if it's empty, it will not be shown in where clause
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
The columns do not correspond with database, it's just show off, what
possibilities buildix has.

```rust
let mut qb = SelectUserBuilder::default();
let (q, _) = query.to_sql::<Postgres>().unwrap();
```

query is now

```sql
SELECT u.name, u.email, u.custom_age AS age, IF(age > 18, true, false) AS is_adult, COALESCE(other, "") AS other FROM user AS u, INNER JOIN order o (o.user_id = u.id) WHERE (priority = ? OR age ISNULL) GROUP BY name, email ORDER BY age ASC
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
SELECT u.name, u.email, u.custom_age AS age, IF(age > 18, true, false) AS is_adult, COALESCE(other, "") AS other FROM user AS u, INNER JOIN order o (o.user_id = u.id) WHERE (priority = ? OR age ISNULL OR (value = ? AND value2 = ?)) GROUP BY name, email ORDER BY age ASC
```

You can see how powerful this filtering is. Not to say that there is more
functionality that helps you to build reliable query builders.


# Execute

Buildix will provide method to execute given query builder, and it detects
whether you want to query single record or multiple records, and it will
set results on query builder instance. If you provide `#[buildix(count)]`
it will also run count queries (for select queries) or in case of update/insert/delete
queries affected rows count. If you do not provide it, the code will not be generated
hence it will be faster.

In the future buildix will also support stream of records, but that's currently
not a priority.

# Delete query builder

**Partially designed.**

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

**Not fully designed yet.**

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

**Not fully designed yet.**


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