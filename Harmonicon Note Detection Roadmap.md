# Harmonicon Note Detection Roadmap

## Goal

Build a **real-time, polyphonic harmonica transcription system** that outputs harmonica tablature (holes + bends), **not generic musical notes**.

---

# Key Insight

Stop treating the problem as **pitch detection**.

Instead, treat it as **harmonica state estimation**.

> "Which harmonica configuration most likely produced this sound?"

This takes advantage of the instrument's physical constraints and is much easier than general Automatic Music Transcription (AMT).

---

# Current Status

Already implemented:

- FFT
- MPM
- YIN
- pYIN

Conclusion:

These algorithms are excellent for **monophonic** signals, but are fundamentally limited for simultaneous harmonica notes.

Do **not** spend more time trying to improve them.

---

# Immediate Next Step

Create a reproducible dataset.

Record:

- Every single note
- Every bend
- Common chords
- Octaves
- Tongue-block intervals
- Real songs

For every recording save:

- WAV
- Detected notes
- Timestamp
- FFT settings

Do **not** modify the detection algorithm until this benchmark exists.

---

# Analyze the Errors

Create a confusion matrix.

Example:

| Played | Detected |
|---------|----------|
| -4 | -4 |
| -5 | -5 |
| -4 -5 | -4 -5 -9 |

Look for:

- recurring phantom notes
- impossible combinations
- systematic failures

---

# Algorithms to Investigate

## 1. Sparse Spectral Reconstruction (Highest Priority)

Build a spectral dictionary of every playable harmonica note.

For each FFT frame solve:

```
Ax ≈ b
```

Where:

- **A** = note templates
- **b** = measured spectrum
- **x** = note activations

Use:

- Non-Negative Least Squares (NNLS)
- L1 sparsity regularization

Advantages:

- Supports multiple simultaneous notes
- Fast
- No neural network required

---

## 2. Harmonica Constraint Solver

After estimating active notes:

Reject impossible combinations.

Examples:

- impossible airflow combinations
- impossible bends
- impossible hole combinations

Prefer:

- fewer notes
- adjacent holes
- physically plausible states

This should greatly reduce phantom detections.

---

## 3. Temporal Smoothing

Instead of analyzing each frame independently:

Use:

- Hidden Markov Model (HMM)
- Viterbi
- Beam Search

Exploit the fact that harmonica states evolve continuously.

---

## 4. Learned Classifier (Long-Term Goal)

Instead of predicting frequencies:

Predict harmonica states directly.

Input:

```
Log-Mel Spectrogram
```

Output:

```
-1
-1'
-2
...
10
```

Each output is an independent sigmoid.

Advantages:

- Learns harmonic interactions automatically
- Learns microphone coloration
- Learns reed characteristics
- Better than generic pitch detection

Possible architecture:

```
Log-Mel
   ↓
CNN
   ↓
GRU / Causal Transformer
   ↓
Sigmoid outputs
```

---

# Avoid

Do not continue investing significant effort into:

- FFT peak picking
- Harmonic Product Spectrum
- YIN
- pYIN
- MPM
- CREPE

These estimate **fundamental frequency**, while the real problem is **multi-source separation**.

---

# Future Experiments

Compare:

- FFT peak detection
- NNLS
- NMF
- Sparse coding
- Small neural network

Using exactly the same benchmark recordings.

---

# Final Goal

The ideal pipeline becomes:

```
Audio
    ↓
STFT
    ↓
Mel Spectrogram
    ↓
Spectral Normalization
    ↓
Sparse Reconstruction (NNLS)
    ↓
Harmonica Constraints
    ↓
Temporal Smoothing
    ↓
Tablature
```

Eventually replace the sparse reconstruction stage with a learned model:

```
Audio
    ↓
Mel Spectrogram
    ↓
CNN + GRU / Causal Transformer
    ↓
Harmonica State Estimation
    ↓
Constraint Solver
    ↓
Tablature
```

This approach is specialized for harmonica and should outperform generic pitch detection algorithms, especially for chords, octaves, and tongue-block techniques.
