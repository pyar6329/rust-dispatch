use anyhow::{Error, Result};
use rust_dispatch::{common_get_value, common_set_value, create_connection};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut conn = create_connection().await?;
    let key: u64 = 1;
    let value: &str = "foo";
    let _ = common_set_value(&mut conn, &key, value).await?;
    let result = common_get_value(&mut conn, &key).await?;
    println!("result = {}", result);
    Ok(())
}
