use anyhow::Result;
use ort::session::Session;

fn main() -> Result<()> {
    let _ = ort::init().with_name("kokoro").commit();
    
    let path = "/Users/ghchinoy/.local/share/kokoro/models/v1.0/model.onnx";
    let session = Session::builder()?.commit_from_file(path)?;
    
    for input in session.inputs() {
        println!("Input: {:?}", input);
    }
    
    Ok(())
}
