use hyper::HeaderMap;
use hyper::header::{self, HeaderName};
//use hyper::Headers;

// header! { (KeepAlive, "Keep-Alive") => [String] }

//pub fn filter_request_headers(headers: &Headers) {
pub fn filter_request_headers(headers: &mut HeaderMap) {
    for value in headers.get_all(header::CONNECTION).iter() {
        println!("Connection: {:?}", value);
    }

//    headers.get::<header::Connection>().and_then(|c| {
//        for c_h in &c.0 {
//
//            match c_h {
//                &header::ConnectionOption::Close => {
//                    let _ = filtered_headers.remove_raw("Close");
//                }
//
//                &header::ConnectionOption::KeepAlive => {
//                    let _ = filtered_headers.remove::<KeepAlive>(); //_raw("Keep-Alive");
//                }
//
//                &header::ConnectionOption::ConnectionHeader(ref o) => {
//                    let _ = filtered_headers.remove_raw(&o);
//                }
//            }
//        }
//
//        Some(c)
//    });

    headers.remove(header::CONNECTION);
    headers.remove(HeaderName::from_static("keep-alive"));
    headers.remove(header::PROXY_AUTHENTICATE);
    headers.remove(header::PROXY_AUTHORIZATION);
    headers.remove(header::TRANSFER_ENCODING);
    headers.remove(header::TRAILER);
    headers.remove(header::UPGRADE);
//    let _ = filtered_headers.remove::<header::Upgrade>();
//
//    filtered_headers
}


// #[test]
// /// Per RFC 2616 Section 13.5.1 - MUST remove hop-by-hop headers
// /// Per RFC 7230 Section 6.1 - MUST remove Connection and Connection option headers
// fn test_filter_frontend_request_headers() {
//     // defining these here only to let me assert
//     header! { (Foo, "Foo") => [String] }
//     header! { (Bar, "Bar") => [String] }
// 
//     let header_vec = vec![
//         ("Transfer-Encoding", "chunked"),
//         ("Host", "example.net"),
//         ("Connection", "Keep-Alive, Foo"),
//         ("Bar", "abc"),
//         ("Foo", "def"),
//         ("Keep-Alive", "timeout=30"),
//         ("Upgrade", "HTTP/2.0, IRC/6.9, RTA/x11, SHTTP/1.3"),
//     ];
// 
//     let mut headers = Headers::new();
// 
//     for (name, value) in header_vec {
//         headers.set_raw(name, value);
//     }
// 
//     let given = filter_frontend_request_headers(&headers);
// 
//     assert_eq!(false, given.has::<header::TransferEncoding>());
//     assert_eq!(true, given.has::<header::Host>());
//     assert_eq!(false, given.has::<header::Connection>());
//     assert_eq!(false, given.has::<Foo>());
//     assert_eq!(true, given.has::<Bar>());
//     assert_eq!(false, given.has::<KeepAlive>());
//     assert_eq!(false, given.has::<header::Upgrade>());
// }
