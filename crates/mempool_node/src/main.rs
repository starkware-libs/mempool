pub mod mempool;
pub mod proxy_mempool;

#[tokio::main]
async fn main() {
    let my_string = "Main function placeholder";
    println!("{}", my_string);
}

#[cfg(test)]
mod proxy_mempool_test;
