Installation and Run:

1. Install Rust in https://www.rust-lang.org/
2. Open terminal in code editor (VsCode) and type cargo new "filename"
3. Open folder that you created
4. Copy the code to src folder and paste in main.rs
5. Go to Cargo.toml
6. Install Dependencies (type below the [dependencies]):
   - iced = { version = "0.10", features = ["tokio"] } (for GUI)
   - sysinfo = "0.29" (gathering Disk Usage information)
   - walkdir = "2.3.2" (Walk through directories files)
   - rayon = "1.7.0" (Parallel processing)
   - serde = { version = "1.0", features = ["derive"] } (produce Json and Csv file format)
   - serde_json = "1.0"
   - csv = "1.1" (export CSV file)
7. Open terminal and type "cargo build"
8. type "cargo run"

Done!
