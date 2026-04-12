# Evaluating TTS Generation

While Kokoro provides fast, reliable, and high-quality speech generation, achieving the optimal emotional tone, pacing, and prosody often requires iterative prompting and voice selection.

Currently, `kokoro-cli` serves as the foundational generation engine. However, the ultimate goal is to move beyond simple generation and introduce a robust **Speech Evaluation Pipeline**. 

## Future Goals: Automated Prosody & Quality Evaluation

Instead of relying solely on subjective community interpretations of voices (e.g., "warm and expressive" vs. "muted and professional"), we aim to implement programmatic evaluation metrics to right-size prompting and provide actionable tuning recommendations.

### Planned Evaluation Metrics

1. **Objective Acoustic Analysis (Technical Prosody)**
   * **Pitch Variability (F0 Contours):** Analyzing the standard deviation of fundamental frequency to quantify "expressiveness" versus "monotone" delivery.
   * **Speaking Rate (WPM/SPM):** Measuring words or syllables per minute to automatically flag "rushing" on long passages (a known issue with Kokoro > 400 tokens).
   * **Pause Duration & Frequency:** Validating whether the engine correctly honors punctuation-induced pauses (commas vs. periods) and identifying hallucinated breath pauses.

2. **Subjective Quality Proxies (Automated MOS)**
   * **Reference-Free Evaluation Models:** Integrating models like [NISQA](https://github.com/gabrielmittag/NISQA) or [WavLM](https://github.com/microsoft/unilm/tree/master/wavlm) to predict Mean Opinion Score (MOS) without requiring human listening tests.
   * **Intelligibility (WER/CER):** Running generated audio through an ASR (Automatic Speech Recognition) model (like Whisper) and comparing the transcript against the original prompt to detect hallucinations, mispronunciations, or slurring.

### Integration with `kokoro-cli`

A future implementation may look like a dedicated `evaluate` command or an `--evaluate` flag on the `speak` command:

```bash
# Example future workflow
kokoro-cli speak "The quick brown fox." --voice af_bella --out test.wav --evaluate

# Example Output:
# [Generation] Success! Audio saved to test.wav
# [Evaluation] Predicted MOS: 4.2/5.0
# [Evaluation] Prosody: High Variance (Expressive)
# [Evaluation] Speaking Rate: 150 WPM (Optimal)
# [Recommendation] Good pacing. No tuning required.
```

By establishing a baseline evaluation process, we can programmatically map Voice IDs and prompting styles to specific emotional traits rather than relying on hardcoded, subjective descriptions.
