
// use md5

// fn create_key(req: &ServiceRequest) -> String {
//     // Get method, path, and query
//     let method = req.method().as_str();
//     let path = req.path();
//     let query = req.query_string();
    
//     let base_key = if query.is_empty() {
//         format!("{}:{}", method, path)
//     } else {
//         format!("{}:{}?{}", method, path, query)
//     };
    
//     // Hash the key using MD5
//     let mut hasher = Md5::new();
//     hasher.update(base_key.as_bytes());
//     let result = hasher.finalize();
    
//     // Convert to hex string
//     format!("cache:{:x}", result)
// }