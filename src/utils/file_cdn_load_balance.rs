use rand::Rng;

/// 根据百分比选择 URL
/// 
/// # Arguments
/// * `primary_url` - 主要 endpoint
/// * `fallback_url` - 回退 endpoint
/// * `primary_percentage` - 主要 endpoint 的流量百分比 (0-100)
pub fn select_cdn_endpoint(primary_endpoint: String, fallback_endpoint: String, primary_percentage: u8) -> String {
    let mut rng = rand::rng();
    let random_value: u8 = rng.random_range(0..100);

    if random_value < primary_percentage {
        primary_endpoint.to_string()
    } else {
        fallback_endpoint.to_string()
    }
}