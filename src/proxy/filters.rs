use http::header::{self, HeaderMap, HeaderName};


pub fn remove_hop_by_hop_headers(headers: &HeaderMap) -> HeaderMap {
    let mut filtered_headers: HeaderMap = headers.clone();
    match headers.get(header::CONNECTION) {
        Some(conn) => {
            let header_value = conn.to_str().unwrap_or("");
            for name in header_value.split(",") {
                let trimmed_name = name.trim();
                let name_bytes = trimmed_name.as_bytes();
                match HeaderName::from_bytes(name_bytes) {
                    Ok(h) => {
                        filtered_headers.remove(h)
                    },
                    Err(_) => None
                };
            }
        },
        None => ()
    }

    filtered_headers.remove(header::CONNECTION);
    filtered_headers.remove(HeaderName::from_static("keep-alive"));
    filtered_headers.remove(header::PROXY_AUTHENTICATE);
    filtered_headers.remove(header::PROXY_AUTHORIZATION);
    filtered_headers.remove(header::TRANSFER_ENCODING);
    filtered_headers.remove(header::TRAILER);
    filtered_headers.remove(header::UPGRADE);

    filtered_headers
}

#[test]
/// Per RFC 2616 Section 13.5.1 - MUST remove hop-by-hop headers
/// Per RFC 7230 Section 6.1 - MUST remove Connection and Connection option headers
fn test_filter_frontend_request_headers() {
    use http::{
        header::{HeaderName, HeaderValue},
        Response
    };

    // let header_vec = vec![
    //     ("Transfer-Encoding", "chunked"),
    //     ("Host", "example.net"),
    //     ("Connection", "Keep-Alive, Foo"),
    //     ("Bar", "abc"),
    //     ("Foo", "def"),
    //     ("Keep-Alive", "timeout=30"),
    //     ("Upgrade", "HTTP/2.0, IRC/6.9, RTA/x11, SHTTP/1.3"),
    // ];

    let keep_alive = HeaderName::from_static("keep-alive");
    let foo = HeaderName::from_static("foo");
    let bar = HeaderName::from_static("bar");
    let baz = HeaderName::from_static("baz");

    let mut headers = HeaderMap::new();
    headers.insert(header::TRANSFER_ENCODING, HeaderValue::from_static("chunked"));
    headers.insert(header::UPGRADE, HeaderValue::from_static("HTTP/2.0, IRC/6.9, RTA/x11, SHTTP/1.3"));
    headers.insert(header::CONNECTION, HeaderValue::from_static("Foo, Bar"));
    headers.insert(keep_alive.clone(), HeaderValue::from_static("timeout=60"));
    headers.insert(foo.clone(), HeaderValue::from_static("Foo Value"));
    headers.insert(bar.clone(), HeaderValue::from_static("Bar Value"));
    headers.insert(baz.clone(), HeaderValue::from_static("Baz Value"));

    let filtered_headers = remove_hop_by_hop_headers(&headers);
    assert_ne!(None, headers.get(header::TRANSFER_ENCODING));
    assert_eq!(None, filtered_headers.get(header::TRANSFER_ENCODING));

    assert_ne!(None, headers.get(header::UPGRADE));
    assert_eq!(None, filtered_headers.get(header::UPGRADE));

    assert_ne!(None, headers.get(keep_alive.clone()));
    assert_eq!(None, filtered_headers.get(keep_alive.clone()));

    // Foo and Bar headers are removed because they are listed in the Connection header
    // Baz header should still be present in the filtered_headers output
    assert_ne!(None, headers.get(foo.clone()));
    assert_eq!(None, filtered_headers.get(foo.clone()));

    assert_ne!(None, headers.get(bar.clone()));
    assert_eq!(None, filtered_headers.get(bar.clone()));

    assert_ne!(None, headers.get(baz.clone()));
    assert_eq!(Some(&HeaderValue::from_static("Baz Value")), filtered_headers.get(baz.clone()));

}
