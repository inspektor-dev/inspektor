#![feature(c_variadic)]
mod opa;
#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Read;
    use std::path::Path;
    use super::*;
    #[test]
    fn it_works() {
        let mut fp = File::open(Path::new("/home/poonai/inspektor/opa/src/policy.wasm")).unwrap();
        let mut buf = Vec::new();
        fp.read_to_end(&mut buf).unwrap();
        let mut policy = opa::OpenPolicy::new(buf).unwrap();
        let result = policy.eval(r#"{"message": "world"}"#.as_bytes()).unwrap();
        println!("result {:?}", String::from_utf8(result));
    }
}
