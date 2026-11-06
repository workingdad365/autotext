fn main() {
    println!("cargo:rerun-if-changed=assets/autotext.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        // 트레이 아이콘은 리소스 ID 1 로 로드함
        res.set_icon_with_id("assets/autotext.ico", "1");
        res.compile().expect("Windows 리소스 컴파일 실패");
    }
}
