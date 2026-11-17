pub const VERSION: &str = match option_env!("AICENTRE_BUILD_VERSION") {
    Some(v) => v,
    None => env!("CARGO_PKG_VERSION"),
};
