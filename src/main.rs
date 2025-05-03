async fn foobar() {
    println!("Back to the future");
}

fn main() {
    println!("Hello");
    let x = foobar();
    println!("What movie ?");
    futures::executor::block_on(x);
}