use fascript::FasRuntime;

#[tokio::main]
async fn main() {
    let code_str = r#"
    //void main() {
    //    os.println("hello world")
    //    os.pppp = (string s) => os.println(s)
    //    os.pppp("aaa")
    //    os.pppp("bbb")
    //}
    //main()
    @retry = 3
    task retry_func() {
        os.println("on retry_func()")
    }
    await retry_func()
    "#;
    let mut runtime = FasRuntime::new();
    //runtime.set_func("my_println", |value: String| println!("{}", value));
    _ = runtime.run(code_str).await;
    //let _ret = runtime.run(code_str).await;
    //println!("{:?}", _ret);
}
