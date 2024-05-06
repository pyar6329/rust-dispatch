use anyhow::{Error, Result};
use rust_dispatch::{common_get_value, common_set_value, create_pool, get_conn, run_query};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let pool = create_pool().await?;
    let mut conn = get_conn(&pool).await?;
    let key: &str = "foo";
    let value: u64 = 1;
    let _ = run_query(common_set_value(&mut conn, &key, &value)).await?;
    println!("aaaaaaaaaaaaa");
    let result = run_query(common_get_value(&mut conn, &key)).await?;
    println!("bbbbbbbbbbbbbb");
    println!("result = {}", result);
    Ok(())
}
