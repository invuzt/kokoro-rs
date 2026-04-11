use anyhow::Result;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use ort::session::builder::GraphOptimizationLevel;
use ort::session::Session;
use misaki_rs::{G2P, Language};

/// Represents the initialized TTS Engine containing the ONNX session
/// and the Misaki G2P parser.
pub struct KokoroEngine {
    onnx_session: Session,
    g2p: G2P,
    vocab: HashMap<char, i64>,
    voices_bin_path: PathBuf,
}

impl KokoroEngine {
    /// Initializes the TTS Engine by loading the Kokoro ONNX model
    /// and the Misaki G2P phonetic dictionaries.
    pub fn new(model_dir: &Path) -> Result<Self> {
        let model_path = model_dir.join("model.onnx");
        let voices_bin_path = model_dir.join("voices.bin");
        let tokens_path = model_dir.join("tokens.txt");

        // WIRING STEP 1: Load the Vocabulary mapping
        println!("  -> [Vocab] Loading tokens.txt...");
        let vocab = Self::load_vocab(&tokens_path)?;

        // WIRING STEP 2: Initialize the Misaki G2P engine
        println!("  -> [Misaki-rs] Initializing G2P engine...");
        let g2p = G2P::new(Language::EnglishUS); // EnglishUS = American English

        // WIRING STEP 3: Initialize the ORT ONNX Session
        println!("  -> [Ort] Loading ONNX model from {:?}...", model_path);
        
        let mut builder = Session::builder()
            .map_err(|e| anyhow::anyhow!("Ort builder error: {:?}", e))?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| anyhow::anyhow!("Ort optimization error: {:?}", e))?
            .with_intra_threads(4)
            .map_err(|e| anyhow::anyhow!("Ort thread error: {:?}", e))?;

        #[cfg(feature = "mac-acceleration")]
        {
            println!("  -> [Ort] Registering CoreML Execution Provider...");
            builder = builder
                .with_execution_providers([ort::execution_providers::CoreMLExecutionProvider::default().build()])
                .map_err(|e| anyhow::anyhow!("Failed to register CoreML: {:?}", e))?;
        }

        let onnx_session = builder
            .commit_from_file(&model_path)
            .map_err(|e| anyhow::anyhow!("Failed to load model.onnx: {:?}", e))?;

        Ok(Self {
            onnx_session,
            g2p,
            vocab,
            voices_bin_path,
        })
    }

    /// Generates raw float32 audio samples from input text.
    pub fn generate_audio(&mut self, text: &str, voice_id: u32, speed: f32) -> Result<Vec<f32>> {
        // STEP 1: Convert text to phonemes using Misaki
        let (phonemes, _) = self.g2p.g2p(text).map_err(|e| anyhow::anyhow!("G2P error: {:?}", e))?;
        println!("  -> [Misaki-rs] Phonemes: {}", phonemes);
        
        // STEP 2: Map phonemes to integer tokens using our vocabulary
        // Kokoro sequences must begin and end with 0 (which acts as BOS/EOS/PAD)
        let mut token_ids: Vec<i64> = Vec::with_capacity(phonemes.chars().count() + 2);
        token_ids.push(0); // BOS
        for c in phonemes.chars() {
            if let Some(&id) = self.vocab.get(&c) {
                token_ids.push(id);
            }
        }
        token_ids.push(0); // EOS
        
        let token_len = token_ids.len();

        // STEP 3: Extract the specific Voice Embedding tensor from voices.bin
        // Each voice consists of 510 styles (based on token length) of 256 floats each.
        // Total floats per voice = 510 * 256 = 130,560 floats = 522,240 bytes.
        println!("  -> [Ort] Extracting voice tensor for ID {}...", voice_id);
        let style_index = std::cmp::min(token_len, 509); 
        let voice_byte_offset = (voice_id as u64) * 522_240; // 510 * 256 * 4 bytes
        let style_byte_offset = voice_byte_offset + ((style_index as u64) * 256 * 4);

        let mut f = File::open(&self.voices_bin_path)?;
        f.seek(SeekFrom::Start(style_byte_offset))?;
        
        let mut style_bytes = vec![0u8; 256 * 4];
        f.read_exact(&mut style_bytes)?;

        // Convert the raw bytes back into f32s (Kokoro weights are little-endian)
        let mut style_vector: Vec<f32> = Vec::with_capacity(256);
        for chunk in style_bytes.chunks_exact(4) {
            let val = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            style_vector.push(val);
        }

        // STEP 4: Prepare Input Tensors for ONNX
        // tokens: [1, sequence_length], Int64
        let tokens_tensor = ort::value::Tensor::from_array(([1, token_len], token_ids))
            .map_err(|e| anyhow::anyhow!("Failed to create tokens tensor: {}", e))?;
            
        // style: [1, 256], Float32
        let style_tensor = ort::value::Tensor::from_array(([1, 256], style_vector))
            .map_err(|e| anyhow::anyhow!("Failed to create style tensor: {}", e))?;
            
        // speed: [1], Float32
        let speed_tensor = ort::value::Tensor::from_array(([1], vec![speed]))
            .map_err(|e| anyhow::anyhow!("Failed to create speed tensor: {}", e))?;

        // STEP 5: Execute the ONNX Graph
        println!("  -> [Ort] Executing ONNX Graph...");
        let outputs = self.onnx_session.run(ort::inputs![
            "tokens" => tokens_tensor,
            "style" => style_tensor,
            "speed" => speed_tensor,
        ])?;

        // STEP 6: Extract the audio float array
        let audio_tensor = outputs["audio"].try_extract_tensor::<f32>()?;
        let audio_samples = audio_tensor.1.to_vec();

        println!("  -> [Ort] Audio generated successfully! ({} samples)", audio_samples.len());
        Ok(audio_samples)
    }

    /// Loads the tokens.txt mapping (e.g., `a 43`) into a HashMap
    fn load_vocab(path: &Path) -> Result<HashMap<char, i64>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut vocab = HashMap::new();

        use std::io::BufRead;
        for line in reader.lines() {
            let line = line?;
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                // The character is the first part, the integer ID is the second
                let c = parts[0].chars().next().unwrap();
                if let Ok(id) = parts[1].parse::<i64>() {
                    vocab.insert(c, id);
                }
            }
        }
        Ok(vocab)
    }
}
