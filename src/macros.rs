#[macro_export]
macro_rules! newtype_id {
    ($name:ident => $table:ident) => {
        #[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
        pub struct $name(i32);

        impl $name {
            pub async fn new(db: &sqlx::PgPool, id: i32) -> anyhow::Result<Self> {
                let query = format!(
                    "SELECT EXISTS(SELECT 1 FROM {} WHERE id = $1)",
                    stringify!($table)
                );
                let result: bool = sqlx::query_scalar(&query).bind(id).fetch_one(db).await?;

                if !result {
                    anyhow::bail!("invalid page id");
                }

                Ok(Self(id))
            }

            pub fn new_unchecked(id: i32) -> Self {
                Self(id)
            }

            pub fn inner(&self) -> i32 {
                self.0
            }
        }
    };
}
