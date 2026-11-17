fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("cannot start the async runtime");
    let code = runtime.block_on(async {
        match aicentre::Ctx::from_process() {
            Ok(ctx) => aicentre::run(&argv, &ctx).await,
            Err(e) => {
                eprintln!("aicentre: {e}");
                1
            }
        }
    });
    runtime.shutdown_background();
    std::process::exit(code);
}
